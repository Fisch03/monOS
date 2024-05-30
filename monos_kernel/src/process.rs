use crate::arch::registers::CR3;
use crate::core_local::CoreLocal;
use crate::gdt::GDT;
use crate::interrupts::InterruptStackFrame;
use crate::mem::{
    active_level_4_table, alloc_frame, alloc_vmem, map_to, physical_mem_offset, translate_addr,
    Frame, MapTo, Mapper, Page, PageSize4K, PageTable, PageTableFlags, PhysicalAddress,
    VirtualAddress,
};

use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use spin::RwLock;

static PROCESSES: RwLock<Vec<Process>> = RwLock::new(Vec::new());
static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
struct Process {
    id: usize,
    // page_table_frame: Frame,
    code_addr: u64,
    stacks: ProcessStacks,
}

//TODO: raise these
const USER_STACK_SIZE: u64 = 0x1000;
const KERNEL_STACK_SIZE: u64 = 0x1000;

#[derive(Debug, Clone)]
struct ProcessStacks {
    user_stack_end: VirtualAddress,
    kernel_stack_end: VirtualAddress,
}

pub fn spawn(entry_point: extern "C" fn()) {
    let pid = Process::new(entry_point);

    let proc = {
        let processes = PROCESSES.read();
        processes.iter().find(|p| p.id == pid).unwrap().clone()
    };

    // set_kernel_stack(&proc);

    // swap to user stack
    unsafe { asm!("swapgs") }; // todo: only do this once per core (in scheduler init?)
    proc.run();
}

fn set_kernel_stack(proc: &Process) {
    let tss = unsafe { &mut *CoreLocal::get().tss.get() };
    tss.privilege_stack_table[0] = proc.stacks.kernel_stack_end;
    CoreLocal::get()
        .kernel_stack
        .set(proc.stacks.kernel_stack_end.as_mut_ptr());
}

static STACK_ADDR: AtomicU64 = AtomicU64::new(0x600_000);
static CODE_ADDR: AtomicU64 = AtomicU64::new(0x400_000);

impl Process {
    fn new(entry_point: extern "C" fn()) -> usize {
        // let page_table_frame = alloc_frame().expect("failed to alloc frame for process page table");
        // let page_table_page = Page::around(unsafe { alloc_vmem(4096).align_up(4096) });
        // unsafe {
        //     map_to(
        //         &page_table_page,
        //         &page_table_frame,
        //         PageTableFlags::PRESENT
        //             | PageTableFlags::WRITABLE
        //             | PageTableFlags::USER_ACCESSIBLE,
        //     )
        //     .expect("failed to map page table page");
        // };

        // let kernel_page_table = active_level_4_table();

        //safety: this page table is invalid right now, but we overwrite it before it's used
        // let process_page_table = page_table_page.start_address().as_mut_ptr::<PageTable>();
        // let process_page_table = unsafe { &mut *process_page_table };
        //
        // process_page_table
        //     .iter_mut()
        //     .zip(kernel_page_table.iter())
        //     .for_each(|(process_entry, kernel_entry)| {
        //         *process_entry = kernel_entry.clone();
        //     });

        // let mut process_mapper = unsafe { Mapper::new(physical_mem_offset(), process_page_table) };

        let kernel_stack_addr = crate::mem::alloc_vmem(KERNEL_STACK_SIZE);
        let kernel_stack_frame = alloc_frame().expect("failed to alloc frame for process stack");
        let kernel_stack_page = Page::around(kernel_stack_addr);
        unsafe {
            // process_mapper.
            map_to(
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
            // process_mapper.
            map_to(
                &user_stack_page,
                &user_stack_frame,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE,
            )
            .expect("failed to map stack page");
        }
        let mut code_addr = CODE_ADDR.fetch_add(0x1000, Ordering::SeqCst);
        let entry_point_phys = translate_addr(VirtualAddress::new(entry_point as u64))
            .expect("failed to translate entry point");
        let code_frame: Frame<PageSize4K> = Frame::around(entry_point_phys);

        code_addr += entry_point_phys.offset_in_page();

        let code_page = Page::around(VirtualAddress::new(code_addr));
        unsafe {
            // process_mapper.
            map_to(
                &code_page,
                &code_frame,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE,
            )
            .expect("failed to map code page");
            // process_mapper.
            // map_to(
            //     &code_page.next(),
            //     &code_frame.next(),
            //     PageTableFlags::PRESENT
            //         | PageTableFlags::WRITABLE
            //         | PageTableFlags::USER_ACCESSIBLE,
            // )
            // .expect("failed to map code page no.2");
        }

        let id = NEXT_PID.fetch_add(1, Ordering::SeqCst);
        let process = Self {
            id,
            // page_table_frame,
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

    //     fn new(entry_point: extern "C" fn()) -> usize {
    //         let page_table_frame = alloc_frame().expect("failed to alloc frame for process page table");
    //         let page_table_page = Page::around(unsafe { alloc_vmem(4096).align_up(4096) });
    //         unsafe {
    //             map_to(
    //                 &page_table_page,
    //                 &page_table_frame,
    //                 PageTableFlags::PRESENT
    //                     | PageTableFlags::WRITABLE
    //                     | PageTableFlags::USER_ACCESSIBLE,
    //             )
    //             .expect("failed to map page table page");
    //         };
    //
    //         let kernel_page_table = active_level_4_table();
    //
    //         //safety: this page table is invalid right now, but we overwrite it before it's used
    //         let process_page_table = page_table_page.start_address().as_mut_ptr::<PageTable>();
    //         let process_page_table = unsafe { &mut *process_page_table };
    //
    //         process_page_table
    //             .iter_mut()
    //             .zip(kernel_page_table.iter())
    //             .for_each(|(process_entry, kernel_entry)| {
    //                 *process_entry = kernel_entry.clone();
    //             });
    //
    //         let mut process_mapper = unsafe { Mapper::new(physical_mem_offset(), process_page_table) };
    //
    //         let entry_point = VirtualAddress::new(entry_point as u64);
    //         let entry_point_phys =
    //             translate_addr(entry_point).expect("failed to translate entry point");
    //         let entry_point_frame: Frame<PageSize4K> = Frame::around(entry_point_phys);
    //
    //         let code_virt = VirtualAddress::new(0x400_000); // TODO: determine this dynamically
    //         let stack_virt = VirtualAddress::new(0x800_000); // TODO: determine this dynamically
    //
    //         let entry_point_page = Page::around(code_virt);
    //         unsafe {
    //             process_mapper
    //                 .map_to(
    //                     &entry_point_page,
    //                     &entry_point_frame,
    //                     PageTableFlags::PRESENT
    //                         | PageTableFlags::WRITABLE  // TODO: is this needed?
    //                         | PageTableFlags::USER_ACCESSIBLE,
    //                 )
    //                 .expect("failed to map entry point page no.1");
    //             process_mapper
    //                 .map_to(
    //                     &entry_point_page.next(),
    //                     &entry_point_frame.next(),
    //                     PageTableFlags::PRESENT
    //                         | PageTableFlags::WRITABLE
    //                         | PageTableFlags::USER_ACCESSIBLE,
    //                 )
    //                 .expect("failed to map entry point page no.2");
    //         };
    //
    //         let stack_page = Page::around(stack_virt);
    //         let stack_frame = alloc_frame().expect("failed to alloc frame for process stack");
    //
    //         unsafe {
    //             process_mapper
    //                 .map_to(
    //                     &stack_page,
    //                     &stack_frame,
    //                     PageTableFlags::PRESENT
    //                         | PageTableFlags::WRITABLE
    //                         | PageTableFlags::USER_ACCESSIBLE,
    //                 )
    //                 .expect("failed to map stack page");
    //         }
    //
    //     }
    //

    fn run(&self) {
        let data = GDT.1.user_data.as_u16();
        let code = GDT.1.user_code.as_u16();

        // let (_, flags) = CR3::read();
        // unsafe {
        //     CR3::write(self.page_table_frame, flags);
        // }

        let (frame, flags) = CR3::read();
        unsafe {
            CR3::write(frame, flags);
        }

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

        // let stack_frame =
        //     InterruptStackFrame::new(entry_point, GDT.1.user_code, 0x200, stack, GDT.1.user_data);

        // unsafe {
        //     stack_frame.iretq();
        // }
    }
}
