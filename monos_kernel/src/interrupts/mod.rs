mod handlers;

mod idt;
pub use idt::init_idt;

use crate::utils::BitField;
use core::arch::asm;
use core::fmt;

pub fn breakpoint() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct VirtualAddress(u64);
impl VirtualAddress {
    fn new(address: u64) -> Self {
        VirtualAddress(address)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
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
    fn from_u16(value: u16) -> Self {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
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
    pub fn new(index: u16, privilege_level: PrivilegeLevel) -> Self {
        let mut selector = 0;
        selector.set_bits(0..2, privilege_level as u16);
        selector.set_bits(3.., index);
        SegmentSelector(selector)
    }

    pub fn privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
    }

    pub fn is_ldt(&self) -> bool {
        self.0.get_bit(2)
    }

    pub fn index(&self) -> u16 {
        self.0.get_bits(3..)
    }

    pub fn current() -> Self {
        let selector: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) selector);
        }
        SegmentSelector(selector)
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SegmentSelector")
            .field("privilege_level", &self.privilege_level())
            // .field("is_ldt", &self.is_ldt())
            .field("index", &self.index())
            .finish()
    }
}

#[repr(C)]
struct InterruptStackFrame {
    instruction_pointer: VirtualAddress,
    code_segment: SegmentSelector,
    _padding1: [u8; 6],
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    stack_segment: SegmentSelector,
    _padding2: [u8; 6],
}

impl fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("InterruptStackFrame")
            .field("instruction_pointer", &self.instruction_pointer)
            .field("code_segment", &self.code_segment)
            .field("cpu_flags", &self.cpu_flags)
            .field("stack_pointer", &self.stack_pointer)
            .field("stack_segment", &self.stack_segment)
            .finish()
    }
}
