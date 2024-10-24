pub mod messaging;
use messaging::{Mailbox, Message, PartialReceiveChannelHandle};

use crate::arch::registers::CR3;
use crate::fs::{fs, File, OpenError, Read};
use crate::gdt::{self, GDT};
use crate::interrupts::without_interrupts;
use crate::mem::{
    alloc_frame, copy_pagetable, create_user_demand_pages, empty_page_table, physical_mem_offset,
    Frame, MapTo, Mapper, Page, PageTableFlags, VirtualAddress, KERNEL_PAGE_TABLE,
};
use monos_std::ProcessId;

use crate::fs::{FileHandle, Path};
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
    mapper: Mapper<'static>,
    page_table_frame: Frame,
    memory: ProcessMemory,
    context_addr: VirtualAddress,
    channels: Vec<Mailbox>,
    file_handles: Vec<File>,
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

const ELF_BYTES: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[derive(Debug)]
pub enum SpawnError {
    FileNotFound,
    NotABinary,
    OpenError(OpenError),
}

pub fn spawn<'p, P: Into<Path<'p>>>(path: P) -> Result<ProcessId, SpawnError> {
    let path = path.into();
    let binary = {
        let file = fs().get(path).unwrap().open().unwrap();

        let mut data = alloc::vec![0u8; file.size()];
        file.read_all(data.as_mut_slice());
        data
    };

    Process::new(&binary.as_slice())
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

    *current = processes.pop_front();

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
    pub fn receive(&mut self, handle: PartialReceiveChannelHandle) -> Option<Message> {
        let mailbox = self.channels.get_mut(handle.own_channel as usize)?;
        mailbox.receive()
    }

    pub fn receive_any(&mut self) -> Option<Message> {
        for mailbox in &mut self.channels {
            if let Some(message) = mailbox.receive() {
                return Some(message);
            }
        }

        None
    }

    //TODO: return result
    pub fn open<'p, P: Into<Path<'p>>>(&mut self, path: P) -> Option<FileHandle> {
        let node = fs().get(path)?;

        if !node.is_file() {
            return None;
        }

        let file = node.open().ok()?;

        self.file_handles.push(file);

        Some(FileHandle::new(self.file_handles.len() as u64 - 1))
    }

    pub fn read(&self, handle: FileHandle, buf: &mut [u8]) -> Option<usize> {
        let handle = self.file_handles.get(handle.as_u64() as usize)?;

        Some(handle.read_all(buf))
    }

    fn new(elf: &[u8]) -> Result<ProcessId, SpawnError> {
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
                    alloc_frame().expect("failed to alloc frame for process stack");

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

                let mut frame = alloc_frame().expect("failed to alloc frame for process");

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
                    frame = alloc_frame().expect("failed to alloc frame for process");
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

            let process = Self {
                id,
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
                file_handles: Vec::new(),
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
