pub mod messaging;
use messaging::{
    add_process_port, ChannelHandle, GenericMessage, Mailbox, MessageType,
    PartialReceiveChannelHandle,
};

use crate::arch::registers::CR3;
use crate::fs::{fs, File, OpenError, Read};
use crate::gdt::{self, GDT};
use crate::interrupts::without_interrupts;
use crate::mem::{
    alloc_frame, copy_pagetable, create_user_demand_pages, empty_page_table, physical_mem_offset,
    Frame, MapTo, Mapper, Page, PageSize4K, PageTableFlags, VirtualAddress, KERNEL_PAGE_TABLE,
};
use alloc::string::{String, ToString};
use monos_std::{
    io::{Seek, SeekMode},
    ProcessId,
};

use crate::fs::{CloseError, FileHandle, Path};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::sync::atomic::{AtomicU32, Ordering};
use object::{Object, ObjectSegment};
use spin::RwLock;

static PROCESS_QUEUE: RwLock<VecDeque<Box<Process>>> = RwLock::new(VecDeque::new());
pub static CURRENT_PROCESS: RwLock<Option<Box<Process>>> = RwLock::new(None);
static NEXT_PID: AtomicU32 = AtomicU32::new(1); // 0 is reserved for the kernel

#[derive(Debug)]
pub struct Process {
    id: ProcessId,
    name: String,
    mapper: Mapper<'static>,
    page_table_frame: Frame,
    memory: ProcessMemory,
    context_addr: VirtualAddress,
    channels: Vec<Mailbox>,
    next_handle: u64,
    file_handles: Vec<(FileHandle, File)>,
    memory_chunks: Vec<MemoryChunk>,
    block_reason: Option<BlockReason>,
}

#[derive(Debug)]
pub enum BlockReason {
    WaitingforSend(ChannelHandle),
}

struct MemoryChunk {
    start_page: Page,
    end_page: Page,
}

impl core::fmt::Debug for MemoryChunk {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemoryChunk")
            .field("start", &self.start_page.start_address())
            .field("end", &self.end_page.end_address())
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ProcessMemory {
    user_stack_end: VirtualAddress,

    kernel_stack_end: VirtualAddress,
    kernel_stack: Vec<u8>,

    heap_start: VirtualAddress,
    heap_size: usize,
}

impl Process {
    pub fn id(&self) -> ProcessId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mapper(&mut self) -> &mut Mapper<'static> {
        &mut self.mapper
    }
}

#[derive(Debug, Clone, Default)]
#[repr(packed)]
pub struct Context {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,

    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub rip: u64,
    pub cs: u64,
    rflags: u64,
    rsp: u64,
    pub ss: u64,
}

const KERNEL_STACK_SIZE: u64 = 0x1000; // 4 KiB

const USER_CODE_START: u64 = 0x200_000; // there is some bootloader stuff at 0x188_00
const USER_STACK_START: u64 = 0x400_000_000_000;
const USER_STACK_SIZE: u64 = 1024 * 1024 * 1; // 1 MiB //TODO: allocations panic the kernel at some point, when fixed increase this
const USER_HEAP_START: u64 = 0x28_000_000_000;
const USER_HEAP_SIZE: u64 = 1024 * 1024 * 128; // 128 MiB
const MEMORY_CHUNK_START: u64 = 0x500_000_000_000;

const ELF_BYTES: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[derive(Debug)]
pub enum SpawnError {
    FileNotFound,
    NotABinary,
    OpenError(OpenError),
}

pub fn spawn<'p, P: Into<Path<'p>>>(path: P) -> Result<ProcessId, SpawnError> {
    let path = path.into();
    let name = path.as_str().to_string();

    let binary = {
        let file = fs().get(path).unwrap().open().unwrap();

        let mut data = alloc::vec![0u8; file.size()];
        file.read_all(data.as_mut_slice());
        data
    };

    Process::new(name, &binary.as_slice())
}

pub fn schedule_next(current_context_addr: VirtualAddress) -> VirtualAddress {
    let mut processes = PROCESS_QUEUE.write();

    // small optimization: if there are no other processes, don't bother switching
    if processes.is_empty() {
        return VirtualAddress::new(0);
    }

    let mut current = CURRENT_PROCESS.write();

    if let Some(mut current) = current.take() {
        current.context_addr = current_context_addr;

        current.page_table_frame = CR3::read().0;

        processes.push_back(current);
    }

    let mut count = 0;

    loop {
        let candidate = processes.pop_front().unwrap();
        if candidate.block_reason.is_none() {
            *current = Some(candidate);
            break;
        } else {
            processes.push_back(candidate);
        }

        count += 1;

        if count == processes.len() {
            break;
        }
    }

    match current.as_ref() {
        Some(current) => {
            gdt::set_kernel_stack(current.memory.kernel_stack_end);
            let (_, flags) = CR3::read();
            unsafe {
                CR3::write(current.page_table_frame, flags);
            }

            current.context_addr
        }
        None => VirtualAddress::new(0),
    }
}

impl Process {
    pub fn block(&mut self, reason: BlockReason) {
        self.block_reason = Some(reason);
    }

    pub fn serve(&mut self, port: &str) -> PartialReceiveChannelHandle {
        let mailbox = Mailbox::new();
        self.channels.push(mailbox);

        add_process_port(port, self.id, self.channels.len() as u16 - 1);

        PartialReceiveChannelHandle {
            own_channel: self.channels.len() as u16 - 1,
        }
    }

    fn receive_chunk(
        &mut self,
        chunk_address: VirtualAddress,
        sender: ProcessId,
    ) -> Option<VirtualAddress> {
        let mut process_queue = PROCESS_QUEUE.write();
        let sender = process_queue
            .iter_mut()
            .find(|p| p.id() == sender)?
            .as_mut();

        let chunk_index = sender
            .memory_chunks
            .iter()
            .position(|chunk| chunk.start_page.start_address() == chunk_address)
            .expect("message contains invalid memory chunk");
        let chunk = sender.memory_chunks.remove(chunk_index);

        let start = self.memory_chunks.last().map_or(
            Page::around(VirtualAddress::new(MEMORY_CHUNK_START)),
            |last| last.end_page.next(),
        );

        let mut current_sender = chunk.start_page;
        let mut current_receiver = start;
        loop {
            let phys = sender
                .mapper
                .translate_addr(current_sender.start_address())
                .expect("failed to translate page");
            let frame = Frame::<PageSize4K>::around(phys);

            sender
                .mapper
                .unmap(&current_sender)
                .expect("failed to unmap page from sender");

            unsafe {
                self.mapper
                    .map_to(
                        &current_receiver,
                        &frame,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::USER_ACCESSIBLE,
                    )
                    .expect("failed to map page to receiver");
            }

            if current_sender == chunk.end_page {
                break;
            }

            current_sender = current_sender.next();
            current_receiver = current_receiver.next();
        }

        self.memory_chunks.push(MemoryChunk {
            start_page: start,
            end_page: current_receiver,
        });

        // crate::println!(
        //     "sent chunk from pid {} at {:#x} to {:#x} -> pid {} at {:#x} to {:#x}",
        //     sender.id().as_u32(),
        //     chunk.start_page.start_address().as_u64(),
        //     chunk.end_page.start_address().as_u64(),
        //     self.id().as_u32(),
        //     start.start_address().as_u64(),
        //     current_receiver.start_address().as_u64()
        // );

        Some(start.start_address())
    }

    pub fn receive(&mut self, handle: PartialReceiveChannelHandle) -> Option<GenericMessage> {
        let mailbox = self.channels.get_mut(handle.own_channel as usize)?;
        let mut msg = mailbox.receive()?;

        if let MessageType::Chunk {
            ref mut address, ..
        } = msg.data
        {
            *address = self
                .receive_chunk(VirtualAddress::new(*address), msg.sender.target_process)?
                .as_u64();
        }

        Some(msg)
    }

    pub fn receive_any(&mut self) -> Option<GenericMessage> {
        for mailbox in &mut self.channels {
            if let Some(mut msg) = mailbox.receive() {
                if let MessageType::Chunk {
                    ref mut address, ..
                } = msg.data
                {
                    *address = self
                        .receive_chunk(VirtualAddress::new(*address), msg.sender.target_process)?
                        .as_u64();
                }

                return Some(msg);
            }
        }

        None
    }

    pub fn request_chunk(&mut self, size: u64) -> Option<VirtualAddress> {
        let start = self.memory_chunks.last().map_or(
            Page::around(VirtualAddress::new(MEMORY_CHUNK_START)),
            |last| last.end_page.next(),
        );

        let end = Page::around(start.start_address() + size);

        let mut current = start;
        loop {
            let frame = alloc_frame("process chunk")?;
            unsafe {
                self.mapper
                    .map_to(
                        &current,
                        &frame,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::USER_ACCESSIBLE,
                    )
                    .ok()?;
            }

            if current == end {
                break;
            }

            current = current.next();
        }

        self.memory_chunks.push(MemoryChunk {
            start_page: start,
            end_page: end,
        });

        crate::println!(
            "requested chunk from {:#x} to {:#x}",
            start.start_address().as_u64(),
            end.start_address().as_u64()
        );

        Some(start.start_address())
    }

    //TODO: return result instead of option
    pub fn open<'p, P: Into<Path<'p>> + core::fmt::Debug>(
        &mut self,
        path: P,
    ) -> Option<FileHandle> {
        let node = fs().get(path)?;

        if !node.is_file() {
            return None;
        }

        let file = node.open().ok()?;

        let handle = FileHandle::new(self.next_handle);
        self.next_handle += 1;
        self.file_handles.push((handle, file));

        Some(handle)
    }

    pub fn close(&mut self, handle: FileHandle) -> Result<(), CloseError> {
        let index = match self.file_handles.iter().position(|(h, _)| *h == handle) {
            Some(index) => index,
            None => return Err(CloseError::NotOpen),
        };

        let (_, file) = self.file_handles.remove(index);
        file.close()
    }

    pub fn seek(&mut self, handle: FileHandle, offset: i64, mode: SeekMode) -> usize {
        let handle = self.file_handles.iter().find(|(h, _)| *h == handle);
        if let Some((_, file)) = handle {
            file.seek(offset, mode)
        } else {
            crate::println!("seek: file handle not found");
            0
        }
    }

    pub fn read(&self, handle: FileHandle, buf: &mut [u8]) -> Option<usize> {
        let handle = self.file_handles.iter().find(|(h, _)| *h == handle)?;

        Some(handle.1.read_all(buf))
    }

    fn new(name: String, elf: &[u8]) -> Result<ProcessId, SpawnError> {
        if &elf[0..4] != &ELF_BYTES {
            return Err(SpawnError::NotABinary);
        }

        let obj = object::File::parse(elf).expect("failed to parse ELF file");

        let kernel_page_table = KERNEL_PAGE_TABLE.get().unwrap();

        let (process_page_table, page_table_frame) = empty_page_table();
        let process_page_table = unsafe { &mut *process_page_table };
        copy_pagetable(kernel_page_table, process_page_table);
        crate::println!(
            "created user page table at {:#x}",
            process_page_table as *mut _ as u64
        );

        let mut process_mapper = unsafe { Mapper::new(physical_mem_offset(), process_page_table) };

        let user_heap_addr = VirtualAddress::new(USER_HEAP_START);
        create_user_demand_pages(&mut process_mapper, user_heap_addr, USER_HEAP_SIZE)
            .expect("failed to create user demand pages");

        without_interrupts(|| {
            let (current_pt_frame, flags) = CR3::read();
            unsafe { CR3::write(page_table_frame, flags) };

            crate::println!("allocating stack");

            let user_stack_start = VirtualAddress::new(USER_STACK_START);
            let mut user_stack_page = Page::around(user_stack_start);
            let user_stack_end = user_stack_page.start_address() + USER_STACK_SIZE;
            let user_stack_end_page = Page::around(user_stack_end);

            user_stack_page = user_stack_page.next(); // skip one page to act as guard page

            loop {
                let user_stack_frame =
                    alloc_frame("process stack").expect("failed to alloc frame for process stack");

                unsafe {
                    process_mapper
                        .map_to(
                            &user_stack_page,
                            &user_stack_frame,
                            PageTableFlags::PRESENT
                                | PageTableFlags::WRITABLE
                                | PageTableFlags::USER_ACCESSIBLE,
                        )
                        .expect("failed to map stack page");
                }

                if user_stack_page == user_stack_end_page {
                    break;
                }

                user_stack_page = user_stack_page.next();
            }

            let code_addr = obj.entry();

            for segment in obj.segments() {
                if segment.address() < USER_CODE_START {
                    panic!("segment address too low");
                }

                let start_addr = VirtualAddress::new(segment.address());
                let end_addr = start_addr + segment.size();

                let mut page = Page::around(start_addr);
                let end_page = Page::around(end_addr.align_up(0x1000));

                let mut frame =
                    alloc_frame("process segment").expect("failed to alloc frame for process");

                loop {
                    unsafe {
                        process_mapper
                            .map_to(
                                &page,
                                &frame,
                                PageTableFlags::PRESENT
                                    | PageTableFlags::WRITABLE
                                    | PageTableFlags::USER_ACCESSIBLE,
                            )
                            .expect("failed to map code page");
                    }

                    if page == end_page {
                        break;
                    }

                    page = page.next();
                    frame =
                        alloc_frame("process segment").expect("failed to alloc frame for process");
                }

                let dest = start_addr.as_mut_ptr::<u8>();
                let src = segment.data().unwrap();

                crate::println!(
                    "copying segment from {:#x} to {:#x} to {:#x}",
                    src.as_ptr() as u64,
                    src.as_ptr() as u64 + src.len() as u64,
                    dest as u64
                );

                unsafe {
                    core::ptr::copy_nonoverlapping(src.as_ptr(), dest, src.len());
                };
            }

            let id = ProcessId(NEXT_PID.fetch_add(1, Ordering::SeqCst));

            let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE as usize);
            let kernel_stack_start = VirtualAddress::from_ptr(kernel_stack.as_ptr());
            let kernel_stack_end = kernel_stack_start + KERNEL_STACK_SIZE;

            let context_addr = kernel_stack_end - core::mem::size_of::<Context>() as u64;

            let context = unsafe { &mut *context_addr.as_mut_ptr::<Context>() };
            *context = Context::default();

            context.rip = code_addr;
            context.rflags = 0x200;

            let data = GDT.1.user_data.as_u16();
            let code = GDT.1.user_code.as_u16();
            context.cs = code as u64;
            context.ss = data as u64;

            context.rsp = user_stack_end.as_u64();

            context.r10 = user_heap_addr.as_u64();
            context.r11 = USER_HEAP_SIZE as u64 - 1;

            crate::println!("name ptr: {:#x}", name.as_ptr() as u64);

            let process = Self {
                id,
                name,
                mapper: process_mapper,
                page_table_frame,
                memory: ProcessMemory {
                    user_stack_end,

                    kernel_stack_end,
                    kernel_stack,

                    heap_start: user_heap_addr,
                    heap_size: USER_HEAP_SIZE as usize - 1,
                },
                context_addr,
                channels: Vec::new(),
                next_handle: 3, // 0, 1, 2 are reserved if we ever do stdin/stdout/stderr
                file_handles: Vec::new(),
                memory_chunks: Vec::new(),
                block_reason: None,
            };

            crate::println!(
                "spawned process {}, entry at {:#x}, stack: {:#x}, heap: {:#x}, pt: {:#x}",
                id,
                code_addr,
                user_stack_end.as_u64(),
                user_heap_addr.as_u64(),
                page_table_frame.start_address().as_u64()
            );

            let mut processes = PROCESS_QUEUE.write();
            processes.push_front(Box::new(process));

            unsafe {
                CR3::write(current_pt_frame, flags);
            }

            Ok(id)
        })
    }
}
