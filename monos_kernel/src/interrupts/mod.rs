mod handlers;

mod apic;
mod idt;

pub fn init() {
    idt::init();
    apic::init();
}

use crate::gdt::SegmentSelector;
use crate::mem::VirtualAddress;
use core::arch::asm;
use core::fmt;

pub fn breakpoint() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

#[repr(C)]
pub struct InterruptStackFrame {
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
