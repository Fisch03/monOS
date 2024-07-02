use super::idt::{IDTEntry, InterruptDescriptorTable};
use super::InterruptStackFrame;
use crate::eprintln;
use crate::gdt::{DOUBLE_FAULT_IST_INDEX, TIMER_IST_INDEX};
use crate::interrupts::apic::LOCAL_APIC;
use crate::mem::{alloc_demand_page, VirtualAddress};

use core::arch::asm;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    APICTimer = 0x20,
    Keyboard = 0x21,
    Mouse = 0x22,

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
    unsafe {
        idt[InterruptIndex::APICTimer.as_usize()]
            .options
            .set_stack_index(TIMER_IST_INDEX);
    }

    idt[InterruptIndex::Keyboard.as_usize()] =
        IDTEntry::new(crate::dev::keyboard::interrupt_handler);
    // unsafe {
    //     idt[InterruptIndex::Keyboard.as_usize()]
    //         .options
    //         .set_stack_index(KEYBOARD_IST_INDEX);
    // }

    idt[InterruptIndex::Mouse.as_usize()] = IDTEntry::new(crate::dev::mouse::interrupt_handler);
    // unsafe {
    //     idt[InterruptIndex::Mouse.as_usize()]
    //         .options
    //         .set_stack_index(MOUSE_IST_INDEX);
    // }

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
    error_code: u64,
) -> ! {
    panic!("double fault\n{:#?}\n{:#?}", error_code, stack_frame);
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
    use x86_64::structures::idt::PageFaultErrorCode;

    let error_code = PageFaultErrorCode::from_bits_truncate(error_code);
    let cr2 = crate::arch::registers::CR2::read();

    if error_code
        == (PageFaultErrorCode::PROTECTION_VIOLATION
            | PageFaultErrorCode::CAUSED_BY_WRITE
            | PageFaultErrorCode::USER_MODE)
    {
        // probably a ondemand page access, try allocating it
        if let Err(msg) = alloc_demand_page(cr2) {
            panic!(
                "page fault\nerror code: {:#?}\ntried accessing memory address: {:#x}\ntried allocating demand page: {}\n{:#?}",
                error_code,
                cr2.as_u64(),
                msg,
                stack_frame,
            );
        }

        // crate::print!("allocated demand page at {:#x}\n", cr2.as_u64());
    } else {
        panic!(
            "page fault\nerror code: {:#?}\ntried accessing memory address: {:#x}\n{:#?}",
            error_code,
            cr2.as_u64(),
            stack_frame
        );
    }
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

#[naked]
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        asm!(
            // disable interrupts
            "cli",
            // save registers onto stack (building a context struct)
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rdi",
            "push rsi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            // get current stack pointer to allow access to context
            "mov rdi, rsp",
            // call handler
            "call {handler}",
            // if the handler returns a new stack pointer, use it
            "cmp rax, 0",
            "je 2f",
            "mov rsp, rax",
            "2:",
            // restore context
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rsi",
            "pop rdi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            // re-enable interrupts
            "sti",
            // all done!
            "iretq",
            handler = sym timer_interrupt_handler_inner,
            options(noreturn),
        );
    }
}

extern "C" fn timer_interrupt_handler_inner(context_addr: u64) -> u64 {
    let context_addr = VirtualAddress::new(context_addr);

    // let context = unsafe { &*(context_addr.as_ptr::<crate::process::Context>()) };
    // let rip = context.rip;
    // crate::println!("before - rip: {:#x}", rip);

    use crate::process;
    let stack_pointer = process::schedule_next(context_addr);

    // if stack_pointer.as_u64() != 0 {
    //     let context = unsafe { &*(stack_pointer.as_ptr::<crate::process::Context>()) };
    //     let rip = context.rip;
    //     crate::println!("after - rip: {:#x}", rip);
    // }

    LOCAL_APIC.get().unwrap().eoi();

    stack_pointer.as_u64()
}

extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    eprintln!("spurious interrupt!");
    LOCAL_APIC.get().unwrap().eoi();
}
