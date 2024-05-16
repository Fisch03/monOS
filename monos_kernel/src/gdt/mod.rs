mod registers;

mod gdt;
use gdt::{GlobalDescriptorTable, SegmentDescriptor};

mod tss;
use tss::TaskStateSegment;

use crate::mem::VirtualAddress;
use crate::utils::BitField;
use core::{arch::asm, ptr::addr_of};
use spin::Lazy;

const STACK_SIZE: usize = 1024 * 8 * 16;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        VirtualAddress::from_ptr(unsafe { addr_of!(STACK) }) + STACK_SIZE as u64
    };

    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Segments)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code = gdt.add_descriptor(SegmentDescriptor::kernel_code());
    let data = gdt.add_descriptor(SegmentDescriptor::kernel_data());
    let tss = gdt.add_descriptor(SegmentDescriptor::tss(&TSS));

    (gdt, Segments { code, data, tss })
});

pub fn init() {
    GDT.0.load();

    unsafe {
        registers::set_cs(GDT.1.code);
        registers::set_ds(GDT.1.data);
        registers::set_es(GDT.1.data);
        registers::set_ss(GDT.1.data);

        asm!("ltr {0:x}", in(reg) GDT.1.tss.as_u16(), options(nostack, preserves_flags))
    }
}

#[derive(Debug, Clone, Copy)]
struct Segments {
    code: SegmentSelector,
    data: SegmentSelector,
    tss: SegmentSelector,
}

///   Segment Selector
/// ┌──┬───────────────┐
/// │ 0│   Privilege   │
/// │ 1│     Level     │
/// ├──┼───────────────┤
/// │ 2│    GDT/LDT    │
/// ├──┼───────────────┤
/// │ 3│               │
/// │ .│               │
/// │ .│     Index     │
/// │ .│               │
/// │15│               │
/// └──┴───────────────┘
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    #[inline]
    pub fn new(index: u16, privilege_level: PrivilegeLevel) -> Self {
        let mut selector = 0;
        selector.set_bits(0..2, privilege_level as u16);
        selector.set_bits(3.., index);
        SegmentSelector(selector)
    }

    #[inline]
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    /// short hand for `SegmentSelector::new(0, PrivilegeLevel::Ring0), but const!
    #[inline]
    pub const fn zero() -> Self {
        SegmentSelector(0)
    }

    pub fn privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
    }

    #[allow(dead_code)]
    pub fn is_ldt(&self) -> bool {
        self.0.get_bit(2)
    }

    pub fn index(&self) -> u16 {
        self.0.get_bits(3..)
    }

    pub fn current() -> Self {
        let selector: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) selector, options(nomem, nostack, preserves_flags));
        }
        SegmentSelector(selector)
    }
}
impl core::fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SegmentSelector")
            .field("privilege_level", &self.privilege_level())
            // .field("is_ldt", &self.is_ldt())
            .field("index", &self.index())
            .finish()
    }
}

#[derive(Debug)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl PrivilegeLevel {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
}
