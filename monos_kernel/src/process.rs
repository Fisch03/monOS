use crate::arch::registers::CR3;
use crate::gdt::GDT;
use crate::interrupts::without_interrupts;
use crate::mem::{
    active_level_4_table, alloc_frame, alloc_vmem, map_to, physical_mem_offset, Frame, MapTo,
    Mapper, Page, PageTable, PageTableFlags, VirtualAddress,
};

use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use object::{Object, ObjectSegment};
use spin::RwLock;

static PROCESSES: RwLock<Vec<Process>> = RwLock::new(Vec::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
struct Process {
    id: usize,
    page_table_frame: Frame,
    code_addr: u64,
    stacks: ProcessStacks,
}

//TODO: raise these
const USER_STACK_SIZE: u64 = 0x1000;
const KERNEL_STACK_SIZE: u64 = 0x1000;

const ELF_BYTES: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[derive(Debug, Clone)]
struct ProcessStacks {
    user_stack_end: VirtualAddress,
    kernel_stack_end: VirtualAddress,
}

pub fn spawn(elf: &[u8]) {
    let pid = Process::new(elf);

    let proc = {
        let processes = PROCESSES.read();
        processes.iter().find(|p| p.id == pid).unwrap().clone()
    };

    //set_kernel_stack(&proc);

    // swap to user stack
    proc.run();
}

// fn set_kernel_stack(proc: &Process) {
//     let tss = unsafe { &mut *CoreLocal::get().tss.get() };
//     tss.privilege_stack_table[0] = proc.stacks.kernel_stack_end;
// }

static STACK_ADDR: AtomicU64 = AtomicU64::new(0x600_000); //TODO: no.

impl Process {
    fn new(elf: &[u8]) -> usize {
        let page_table_frame = alloc_frame().expect("failed to alloc frame for process page table");
        let page_table_page = Page::around(alloc_vmem(4096));
        unsafe {
            map_to(
                &page_table_page,
                &page_table_frame,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE,
            )
            .expect("failed to map page table page");
        };

        let kernel_page_table = active_level_4_table();

        //safety: this page table is invalid right now, but we overwrite it before it's used
        let process_page_table = page_table_page.start_address().as_mut_ptr::<PageTable>();
        let process_page_table = unsafe { &mut *process_page_table };

        process_page_table
            .iter_mut()
            .zip(kernel_page_table.iter())
            .for_each(|(process_entry, kernel_entry)| {
                *process_entry = kernel_entry.clone();
            });

        let mut process_mapper = unsafe { Mapper::new(physical_mem_offset(), process_page_table) };

        let kernel_stack_addr = crate::mem::alloc_vmem(KERNEL_STACK_SIZE);
        let kernel_stack_frame = alloc_frame().expect("failed to alloc frame for process stack");
        let kernel_stack_page = Page::around(kernel_stack_addr);
        unsafe {
            process_mapper
                .map_to(
                    &kernel_stack_page,
                    &kernel_stack_frame,
                    PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE,
                )
                .expect("failed to map stack page");
        }

        let user_stack_addr = VirtualAddress::new(STACK_ADDR.fetch_add(0x1000, Ordering::SeqCst));
        let user_stack_frame = alloc_frame().expect("failed to alloc frame for process stack");
        let user_stack_page = Page::around(user_stack_addr);
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

        assert_eq!(&elf[0..4], &ELF_BYTES, "not an ELF file");
        let obj = object::File::parse(elf).expect("failed to parse ELF file");
        let code_addr = obj.entry();

        for segment in obj.segments() {
            let start_addr = VirtualAddress::new(segment.address());
            let end_addr = start_addr + segment.size();

            //TODO: check address bounds

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

            let mut dest = start_addr.as_mut_ptr::<u8>();
            let mut src = segment.data().unwrap();

            let (current_pt_frame, flags) = CR3::read();
            without_interrupts(|| {
                unsafe {
                    CR3::write(page_table_frame, flags);
                }

                // horribleness. i should really figure out how to compile userspace programs at an
                // offset
                if start_addr == VirtualAddress::new(0) {
                    dest = unsafe { dest.add(1) };
                    src = &src[1..];
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
        let process = Self {
            id,
            page_table_frame,
            code_addr,
            stacks: ProcessStacks {
                user_stack_end: user_stack_addr + USER_STACK_SIZE,
                kernel_stack_end: kernel_stack_addr + KERNEL_STACK_SIZE,
            },
        };

        let mut processes = PROCESSES.write();
        processes.push(process);

        id
    }

    fn run(&self) {
        let data = GDT.1.user_data.as_u16();
        let code = GDT.1.user_code.as_u16();

        let (_, flags) = CR3::read();
        unsafe {
            CR3::write(self.page_table_frame, flags);
        }

        // let (frame, flags) = CR3::read();
        // unsafe {
        //     CR3::write(frame, flags);
        // }

        unsafe {
            crate::interrupts::disable();
            asm!(
                "push rax",
                "push rsi",
                "push 0x200",
                "push rdx",
                "push rdi",
                "iretq",
                in("rax") data,
                in("rsi") self.stacks.user_stack_end.as_u64(),
                in("rdx") code,
                in("rdi") self.code_addr,
            );
        }

        // let stack_frame = InterruptStackFrame::new(
        //     VirtualAddress::new(self.code_addr),
        //     GDT.1.user_code,
        //     0x200,
        //     self.stacks.user_stack_end,
        //     GDT.1.user_data,
        // );

        // unsafe {
        //     stack_frame.iretq();
        // }
    }
}
