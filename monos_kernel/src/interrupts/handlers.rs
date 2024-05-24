use super::idt::{IDTEntry, InterruptDescriptorTable};
use super::InterruptStackFrame;
use crate::eprintln;
use crate::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::interrupts::apic::LOCAL_APIC;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    APICTimer = 0x20,
    Keyboard = 0x21,

    SpuriousInterrupt = 0xFF,
}
impl InterruptIndex {
    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

pub fn attach_handlers(idt: &mut InterruptDescriptorTable) {
    idt.division_error = IDTEntry::new(division_error_handler);
    idt.debug = IDTEntry::new(debug_handler);
    idt.non_maskable_interrupt = IDTEntry::new(non_maskable_interrupt_handler);
    idt.breakpoint = IDTEntry::new(breakpoint_handler);
    idt.invalid_opcode = IDTEntry::new(invalid_opcode_handler);
    idt.device_not_available = IDTEntry::new(device_not_available_handler);

    idt.double_fault = IDTEntry::new(double_fault_handler);
    unsafe {
        idt.double_fault
            .options
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt.invalid_tss = IDTEntry::new(invalid_tss_handler);
    idt.segment_not_present = IDTEntry::new(segment_not_present_handler);
    idt.stack_segment_fault = IDTEntry::new(stack_segment_fault_handler);
    idt.general_protection_fault = IDTEntry::new(general_protection_fault_handler);
    idt.page_fault = IDTEntry::new(page_fault_handler);
    idt.x87_floating_point = IDTEntry::new(x87_floating_point_handler);
    idt.alignment_check = IDTEntry::new(alignment_check_handler);
    idt.simd_floating_point = IDTEntry::new(simd_floating_point_handler);
    idt.virtualization_exception = IDTEntry::new(virtualization_exception_handler);
    idt.control_protection_exception = IDTEntry::new(control_protection_exception_handler);
    idt.hypervisor_injection_exception = IDTEntry::new(hypervisor_injection_exception_handler);
    idt.vmm_communication_exception = IDTEntry::new(vmm_communication_exception_handler);
    idt.security_exception = IDTEntry::new(security_exception_handler);

    idt[InterruptIndex::APICTimer.as_usize()] = IDTEntry::new(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()] =
        IDTEntry::new(crate::dev::keyboard::interrupt_handler);
    idt[InterruptIndex::SpuriousInterrupt.as_usize()] = IDTEntry::new(spurious_interrupt_handler);
}

macro_rules! irq_handler {
    ($name:ident) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            eprintln!("unhandled interrupt: {}", stringify!($name));
            eprintln!("{:#?}", stack_frame);
        }
    };
}

macro_rules! irq_handler_err {
    ($name:ident) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, error_code: u64) {
            eprintln!("unhandled interrupt: {}", stringify!($name));
            eprintln!("error code: {:#b}", error_code);
            eprintln!("{:#?}", stack_frame);
        }
    };
}

irq_handler!(division_error_handler);
irq_handler!(debug_handler);
irq_handler!(non_maskable_interrupt_handler);

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    eprintln!("breakpoint interrupt!");
}

irq_handler!(invalid_opcode_handler);
irq_handler!(device_not_available_handler);

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("double fault\n{:#?}", stack_frame);
}

irq_handler_err!(invalid_tss_handler);
irq_handler_err!(segment_not_present_handler);
irq_handler_err!(stack_segment_fault_handler);

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "general protection fault\nerror code: {:#?}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "page fault\nerror code: {:#x}\n{:#?}",
        error_code, stack_frame
    );
}

irq_handler!(x87_floating_point_handler);
irq_handler_err!(alignment_check_handler);

// machine check exception

irq_handler!(simd_floating_point_handler);
irq_handler!(virtualization_exception_handler);
irq_handler_err!(control_protection_exception_handler);
irq_handler!(hypervisor_injection_exception_handler);
irq_handler_err!(vmm_communication_exception_handler);
irq_handler_err!(security_exception_handler);

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // crate::print!(".");
    LOCAL_APIC.get().unwrap().eoi();
}

extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    eprintln!("spurious interrupt!");
    LOCAL_APIC.get().unwrap().eoi();
}
