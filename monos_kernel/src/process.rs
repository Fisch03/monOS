use crate::arch::registers::CR3;
use crate::gdt::{self, GDT};
use crate::interrupts::without_interrupts;
use crate::mem::{
    active_level_4_table, alloc_frame, alloc_vmem, copy_pagetable, empty_page_table, map_to,
    physical_mem_offset, Frame, MapTo, Mapper, Page, PageTable, PageTableFlags, VirtualAddress,
};

use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};
use object::{Object, ObjectSegment};
use spin::RwLock;

static PROCESS_QUEUE: RwLock<VecDeque<Box<Process>>> = RwLock::new(VecDeque::new());
static CURRENT_PROCESS: RwLock<Option<Box<Process>>> = RwLock::new(None);
static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct Process {
    #[allow(dead_code)]
    id: usize,
    page_table_frame: Frame,
    stacks: ProcessStacks,
    heap_start: VirtualAddress,
    heap_size: usize,
    context_addr: VirtualAddress,
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
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

const USER_CODE_START: u64 = 0x10000;
const KERNEL_STACK_SIZE: u64 = 0x1000;

const ELF_BYTES: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[derive(Debug, Clone)]
struct ProcessStacks {
    user_stack_end: VirtualAddress,
    kernel_stack_end: VirtualAddress,
    kernel_stack: Vec<u8>,
}

pub fn spawn(elf: &[u8]) {
    Process::new(elf);
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
            gdt::set_kernel_stack(current.stacks.kernel_stack_end);

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
    fn new(elf: &[u8]) -> usize {
        assert_eq!(&elf[0..4], &ELF_BYTES, "not an ELF file");
        let obj = object::File::parse(elf).expect("failed to parse ELF file");

        let mut free_start = VirtualAddress::new(obj.segments().fold(0, |free_start, segment| {
            free_start.max(segment.address() + segment.size())
        }));

        free_start = free_start.align_up(0x1000) + 0x1000;

        let kernel_page_table = active_level_4_table(); // TODO: this may break if we are currently running a process

        let (process_page_table, page_table_frame) = empty_page_table();
        let process_page_table = unsafe { &mut *process_page_table };
        copy_pagetable(kernel_page_table, process_page_table);

        let mut process_mapper = unsafe { Mapper::new(physical_mem_offset(), process_page_table) };

        let user_stack_addr = free_start.clone();
        let mut user_stack_size = 0;
        let mut user_stack_page = Page::around(user_stack_addr);
        let user_stack_addr = user_stack_page.start_address();
        for _ in 0..10 {
            let user_stack_frame = alloc_frame().expect("failed to alloc frame for process stack");

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
            user_stack_page = user_stack_page.next();

            free_start += 0x1000;
            user_stack_size += 0x1000;
        }
        let user_stack_end = user_stack_addr + user_stack_size;

        free_start += 0x4000;

        let user_heap_addr = free_start.clone();
        let mut user_heap_size = 0;
        let mut user_heap_page = Page::around(user_heap_addr);
        let user_heap_addr = user_heap_page.start_address();
        for _ in 0..100 {
            let user_heap_frame = alloc_frame().expect("failed to alloc frame for process heap");

            unsafe {
                process_mapper
                    .map_to(
                        &user_heap_page,
                        &user_heap_frame,
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::USER_ACCESSIBLE,
                    )
                    .expect("failed to map heap page");
            }
            user_heap_page = user_heap_page.next();

            free_start += 0x1000;
            user_heap_size += 0x1000;
        }

        let code_addr = obj.entry();

        for segment in obj.segments() {
            if segment.address() < USER_CODE_START {
                panic!("segment address too low");
            }
            let start_addr = VirtualAddress::new(segment.address());
            let end_addr = start_addr + segment.size();

            let mut page = Page::around(start_addr);
            let end_page = Page::around(end_addr);

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
            let (current_pt_frame, flags) = CR3::read();
            without_interrupts(|| {
                unsafe {
                    CR3::write(page_table_frame, flags);
                }

                unsafe {
                    core::ptr::copy_nonoverlapping(src.as_ptr(), dest, src.len());
                }

                unsafe {
                    CR3::write(current_pt_frame, flags);
                }
            });
        }

        let id = NEXT_PID.fetch_add(1, Ordering::SeqCst);

        let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE as usize);
        let kernel_stack_start = VirtualAddress::from_ptr(kernel_stack.as_ptr());
        let kernel_stack_end = kernel_stack_start + KERNEL_STACK_SIZE;

        let context_addr = kernel_stack_end - core::mem::size_of::<Context>() as u64;
        without_interrupts(|| {
            let (current_pt_frame, flags) = CR3::read();
            unsafe {
                CR3::write(page_table_frame, flags);
            }

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
            context.r11 = user_heap_size as u64;

            unsafe {
                CR3::write(current_pt_frame, flags);
            }
        });

        let process = Self {
            id,
            page_table_frame,
            stacks: ProcessStacks {
                user_stack_end,
                kernel_stack_end,
                kernel_stack,
            },
            heap_start: user_heap_addr,
            heap_size: user_heap_size - 1,
            context_addr,
        };

        crate::println!("spawned process {:#?}", process);

        let mut processes = PROCESS_QUEUE.write();
        processes.push_front(Box::new(process));

        id
    }
}
