use crate::dbg;
use crate::mem::VirtualAddress;

use core::mem::size_of;
use core::ptr::addr_of;
use spin::Once;
use x86_64::instructions::segmentation::{Segment, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

const STACK_SIZE: usize = 1024 * 8 * 16;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Once<TaskStateSegment> = Once::new();
pub static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();

pub fn init() {
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { addr_of!(STACK) }) + STACK_SIZE as u64
        };

        dbg!(tss)
    });

    let gdt = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.append(Descriptor::kernel_code_segment());
        let tss = gdt.append(Descriptor::tss_segment(tss));
        (gdt, Selectors { tss, code })
    });

    gdt.0.load();
    unsafe {
        CS::set_reg(gdt.1.code);
        load_tss(gdt.1.tss);
    }
}

#[derive(Debug)]
pub struct Selectors {
    tss: SegmentSelector,
    code: SegmentSelector,
}

// #[derive(Debug, Clone, Copy)]
// #[repr(C, packed(4))]
// struct TaskStateSegment {
//     _reserved: u32,
//     privilege_stack_table: [VirtualAddress; 3],
//     _reserved2: u64,
//     interrupt_stack_table: [VirtualAddress; 7],
//     _reserved3: u64,
//     _reserved4: u16,
//     iomap_base: u16,
// }
//
// impl TaskStateSegment {
//     #[inline]
//     const fn new() -> Self {
//         Self {
//             privilege_stack_table: [VirtualAddress::zero(); 3],
//             interrupt_stack_table: [VirtualAddress::zero(); 7],
//             iomap_base: size_of::<Self>() as u16,
//
//             _reserved: 0,
//             _reserved2: 0,
//             _reserved3: 0,
//             _reserved4: 0,
//         }
//     }
// }
