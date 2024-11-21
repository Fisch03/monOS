pub mod handlers;
pub use handlers::InterruptIndex;

pub mod apic;
mod idt;

use crate::gdt::SegmentSelector;
use crate::mem::VirtualAddress;
use crate::utils::BitField;
use core::{arch::asm, fmt};

// should be called as early as possible
pub fn init_idt() {
    idt::init();
}

// this requires memory to be initialized
pub fn init_apic() {
    apic::init();
}

#[inline]
pub fn enable() {
    unsafe {
        asm!("sti", options(preserves_flags, nostack));
    }
}

#[inline]
pub fn disable() {
    unsafe {
        asm!("cli", options(preserves_flags, nostack));
    }
}

#[inline]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let flags: u64;
    unsafe {
        asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags));
    }
    let interrupts_enabled = flags.get_bit(9);

    if interrupts_enabled {
        disable();
    }

    let result = f();

    if interrupts_enabled {
        enable();
    }

    result
}

#[inline]
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

impl InterruptStackFrame {
    pub fn new(
        instruction_pointer: VirtualAddress,
        code_segment: SegmentSelector,
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        stack_segment: SegmentSelector,
    ) -> Self {
        Self {
            instruction_pointer,
            code_segment,
            _padding1: [0; 6],
            cpu_flags,
            stack_pointer,
            stack_segment,
            _padding2: [0; 6],
        }
    }

    #[inline]
    pub unsafe fn iretq(&self) -> ! {
        unsafe {
            asm!(
                "push {stack_segment:r}",
                "push {new_stack_pointer:r}",
                "push {cpu_flags}",
                "push {code_segment:r}",
                "push {new_instruction_pointer:r}",
                "iretq",

                stack_segment = in(reg) self.stack_segment.as_u16(),
                new_stack_pointer = in(reg) self.stack_pointer.as_u64(),
                cpu_flags = in(reg) self.cpu_flags,
                code_segment = in(reg) self.code_segment.as_u16(),
                new_instruction_pointer = in(reg) self.instruction_pointer.as_u64(),
                options(noreturn)
            )
        }
    }
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
