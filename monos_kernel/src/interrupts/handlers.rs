use super::idt::{IDTEntry, InterruptDescriptorTable};
use super::InterruptStackFrame;
use crate::eprintln;

pub fn attach_handlers(idt: &mut InterruptDescriptorTable) {
    idt.breakpoint = IDTEntry::new(breakpoint_handler);
    idt.double_fault = IDTEntry::new(double_fault_handler);
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    eprintln!("breakpoint interrupt!\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("kernel exception: double fault\n{:#?}", stack_frame);
}
