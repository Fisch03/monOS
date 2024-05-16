use super::handlers::attach_handlers;
use super::{InterruptStackFrame, PrivilegeLevel, SegmentSelector, VirtualAddress};

use crate::utils::BitField;
use core::arch::asm;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;
use spin::Once;

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
    IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        attach_handlers(&mut idt);
        idt
    })
    .load();
}

pub type IDTHandlerFunction = extern "x86-interrupt" fn(_: InterruptStackFrame);
pub type IDTHandlerFunctionErrCode = extern "x86-interrupt" fn(_: InterruptStackFrame, _: u64);
pub type IDTHandlerFunctionDiverging = extern "x86-interrupt" fn(_: InterruptStackFrame) -> !;
pub type IDTHandlerFunctionErrCodeDiverging =
    extern "x86-interrupt" fn(_: InterruptStackFrame, _: u64) -> !;

/// a trait for interrupt handler functions.
///
/// safety: must return a valid address to the handler function.
pub unsafe trait HandlerFn {
    fn address(self) -> VirtualAddress;
}

macro_rules! impl_handler_fn {
    ($fn:ty) => {
        unsafe impl HandlerFn for $fn {
            fn address(self) -> VirtualAddress {
                VirtualAddress::new(self as u64)
            }
        }
    };
}

impl_handler_fn!(IDTHandlerFunction);
impl_handler_fn!(IDTHandlerFunctionErrCode);
impl_handler_fn!(IDTHandlerFunctionDiverging);
impl_handler_fn!(IDTHandlerFunctionErrCodeDiverging);

#[derive(Debug, Clone)]
#[repr(C, packed(2))]
struct IDTPointer {
    limit: u16,
    base: VirtualAddress,
}

impl IDTPointer {
    /// load the table at the pointer adress.
    ///
    /// safety: the pointer must point to a valid IDT and have the correct limit.
    unsafe fn load(&self) {
        asm!("lidt [{}]", in(reg) self, options(nostack, preserves_flags));
    }
}

// https://wiki.osdev.org/Exceptions
#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    pub division_error: IDTEntry<IDTHandlerFunction>,
    pub debug: IDTEntry<IDTHandlerFunction>,
    pub non_maskable_interrupt: IDTEntry<IDTHandlerFunction>,
    pub breakpoint: IDTEntry<IDTHandlerFunction>,
    pub overflow: IDTEntry<IDTHandlerFunction>,
    pub bound_range_exceeded: IDTEntry<IDTHandlerFunction>,
    pub invalid_opcode: IDTEntry<IDTHandlerFunction>,
    pub device_not_available: IDTEntry<IDTHandlerFunction>,
    pub double_fault: IDTEntry<IDTHandlerFunctionErrCodeDiverging>,
    _coprocessor_segment_overrun: IDTEntry<IDTHandlerFunction>,
    pub invalid_tss: IDTEntry<IDTHandlerFunctionErrCode>,
    pub segment_not_present: IDTEntry<IDTHandlerFunctionErrCode>,
    pub stack_segment_fault: IDTEntry<IDTHandlerFunctionErrCode>,
    pub general_protection_fault: IDTEntry<IDTHandlerFunctionErrCode>,
    pub page_fault: IDTEntry<IDTHandlerFunctionErrCode>, // TODO: handler fn with page fault err codes
    _reserved: IDTEntry<IDTHandlerFunction>,
    pub x87_floating_point: IDTEntry<IDTHandlerFunction>,
    pub alignment_check: IDTEntry<IDTHandlerFunctionErrCode>,
    pub machine_check: IDTEntry<IDTHandlerFunction>,
    pub simd_floating_point: IDTEntry<IDTHandlerFunction>,
    pub virtualization_exception: IDTEntry<IDTHandlerFunction>,
    pub control_protection_exception: IDTEntry<IDTHandlerFunctionErrCode>,
    _reserved2: [IDTEntry<IDTHandlerFunction>; 6],
    pub hypervisor_injection_exception: IDTEntry<IDTHandlerFunction>,
    pub vmm_communication_exception: IDTEntry<IDTHandlerFunctionErrCode>,
    pub security_exception: IDTEntry<IDTHandlerFunctionErrCode>,
    _reserved3: IDTEntry<IDTHandlerFunction>,
}

impl InterruptDescriptorTable {
    #[inline]
    pub const fn new() -> Self {
        Self {
            division_error: IDTEntry::new_empty(),
            debug: IDTEntry::new_empty(),
            non_maskable_interrupt: IDTEntry::new_empty(),
            breakpoint: IDTEntry::new_empty(),
            overflow: IDTEntry::new_empty(),
            bound_range_exceeded: IDTEntry::new_empty(),
            invalid_opcode: IDTEntry::new_empty(),
            device_not_available: IDTEntry::new_empty(),
            double_fault: IDTEntry::new_empty(),
            _coprocessor_segment_overrun: IDTEntry::new_empty(),
            invalid_tss: IDTEntry::new_empty(),
            segment_not_present: IDTEntry::new_empty(),
            stack_segment_fault: IDTEntry::new_empty(),
            general_protection_fault: IDTEntry::new_empty(),
            page_fault: IDTEntry::new_empty(),
            _reserved: IDTEntry::new_empty(),
            x87_floating_point: IDTEntry::new_empty(),
            alignment_check: IDTEntry::new_empty(),
            machine_check: IDTEntry::new_empty(),
            simd_floating_point: IDTEntry::new_empty(),
            virtualization_exception: IDTEntry::new_empty(),
            control_protection_exception: IDTEntry::new_empty(),
            _reserved2: [IDTEntry::new_empty(); 6],
            hypervisor_injection_exception: IDTEntry::new_empty(),
            vmm_communication_exception: IDTEntry::new_empty(),
            security_exception: IDTEntry::new_empty(),
            _reserved3: IDTEntry::new_empty(),
        }
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        let ptr = IDTPointer {
            base: VirtualAddress::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };
        unsafe {
            ptr.load();
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IDTEntry<HandlerFnType>
where
    HandlerFnType: HandlerFn,
{
    pointer_lower: u16,
    gdt_selector: SegmentSelector,
    pub options: IDTEntryOptions,
    pointer_middle: u16,
    pointer_upper: u32,
    _reserved: u32,

    handler_fn_type: PhantomData<HandlerFnType>,
}

impl<T: HandlerFn> IDTEntry<T> {
    /// create a new IDT entry.
    ///
    /// safety: the `handler_address` must be an adress to a valid interrupt handler.
    #[inline]
    pub fn new(handler_fn: T) -> Self {
        let handler_address = handler_fn.address().as_u64();
        Self {
            gdt_selector: SegmentSelector::current(),
            options: IDTEntryOptions::new(),
            pointer_lower: handler_address as u16,
            pointer_middle: (handler_address >> 16) as u16,
            pointer_upper: (handler_address >> 32) as u32,
            _reserved: 0,

            handler_fn_type: PhantomData,
        }
    }

    #[inline]
    pub const fn new_empty() -> Self {
        Self {
            gdt_selector: SegmentSelector::zero(),
            options: IDTEntryOptions::new_empty(),
            pointer_lower: 0,
            pointer_middle: 0,
            pointer_upper: 0,
            _reserved: 0,

            handler_fn_type: PhantomData,
        }
    }
}

///  IDT Entry Options
///
/// ┌──┬───────────────┐
/// │ 0│   Interrupt   │
/// │ 1│  Stack Table  │
/// │ 2│     Index     │
/// ├──┼───────────────┤
/// │ 3│               │
/// │ 4│               │
/// │ 5│   Reserved    │
/// │ 6│               │
/// │ 7│               │
/// ├──┼───────────────┤
/// │ 8│Interrupt/Trap │
/// ├──┼───────────────┤
/// │ 9│               │
/// │10│   always 1    │
/// │11│               │
/// ├──┼───────────────┤
/// │12│   always 0    │
/// ├──┼───────────────┤
/// │13│  Descriptor   │
/// │14│Privilege Level│
/// ├──┼───────────────┤
/// │15│   Present     │
/// └──┴───────────────┘
#[derive(Clone, Copy)]
pub struct IDTEntryOptions(u16);

impl IDTEntryOptions {
    const STACK_INDEX: Range<usize> = 0..3;
    const INTERRUPT_TRAP: usize = 8;
    const DPL: Range<usize> = 13..15;
    const PRESENT: usize = 15;

    #[inline]
    fn new() -> Self {
        *Self::new_empty().set_present(true).set_interrupts(false)
    }

    #[inline]
    const fn new_empty() -> Self {
        Self(0b1110_0000_0000)
    }

    /// set the stack index.
    ///
    /// safety: the `index` must be a valid index
    #[allow(dead_code)]
    pub unsafe fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(Self::STACK_INDEX, index + 1);
        self
    }

    #[allow(dead_code)]
    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0.set_bits(Self::DPL, dpl);
        self
    }

    #[allow(dead_code)]
    pub fn set_interrupts(&mut self, enable: bool) -> &mut Self {
        self.0.set_bit(Self::INTERRUPT_TRAP, enable);
        self
    }

    #[allow(dead_code)]
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(Self::PRESENT, present);
        self
    }
}

impl fmt::Debug for IDTEntryOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IDTEntryOptions")
            .field(
                "stack_index",
                &self.0.get_bits(IDTEntryOptions::STACK_INDEX),
            )
            .field(
                "interrupts",
                &self.0.get_bit(IDTEntryOptions::INTERRUPT_TRAP),
            )
            .field("dpl", &self.0.get_bits(IDTEntryOptions::DPL))
            .field("present", &self.0.get_bit(IDTEntryOptions::PRESENT))
            .finish()
    }
}
