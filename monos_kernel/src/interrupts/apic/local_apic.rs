use crate::mem::VirtualAddress;
use core::ptr::{read_volatile, write_volatile};

use crate::interrupts::InterruptIndex;

#[allow(dead_code)]
const SPURIOS_INTERRUPT_ENABLE: u32 = 1 << 8;
#[allow(dead_code)]
const SPURIOS_INTERRUPT_FCC: u32 = 1 << 9;

#[allow(dead_code)]
const TIMER_ONE_SHOT: u32 = 0b00 << 17;
const TIMER_PERIODIC: u32 = 0b01 << 17;
#[allow(dead_code)]
const TIMER_TSC_DEADLINE: u32 = 0b10 << 17;

#[repr(u32)]
#[allow(dead_code)]
pub enum LocalAPICField {
    Id = 0x20,
    Version = 0x30,
    TaskPriority = 0x80,
    ArbitrationPriority = 0x90,
    ProcessorPriority = 0xA0,
    EOI = 0xB0,
    RemoteRead = 0xC0,
    LogicalDestination = 0xD0,
    DestinationFormat = 0xE0,

    SpuriousInterruptVector = 0xF0,

    InService = 0x100,
    TriggerMode = 0x180,
    InterruptRequest = 0x200,
    ErrorStatus = 0x280,
    InterruptCommandLow = 0x300,
    InterruptCommandHigh = 0x310,
    TimerLVT = 0x320,
    ThermalLVT = 0x330,
    PerformanceCounterLVT = 0x340,
    LINT0LVT = 0x350,
    LINT1LVT = 0x360,
    ErrorLVT = 0x370,

    TimerInitialCount = 0x380,
    TimerCurrentCount = 0x390,
    TimerDivideConfig = 0x3E0,
}

pub struct LocalAPIC {
    base_address: VirtualAddress,
}

impl LocalAPIC {
    pub fn new(base_address: VirtualAddress) -> Self {
        let mut local_apic = Self { base_address };

        crate::dbg!(local_apic.id());
        crate::dbg!(local_apic.version());

        local_apic.write(
            LocalAPICField::SpuriousInterruptVector,
            InterruptIndex::SpuriousInterrupt.as_u32() | SPURIOS_INTERRUPT_ENABLE,
        );

        local_apic.write(
            LocalAPICField::TimerLVT,
            InterruptIndex::APICTimer.as_u32() | TIMER_PERIODIC,
        );

        local_apic
    }

    pub fn read(&self, field: LocalAPICField) -> u32 {
        let addr = self.base_address.as_u64() + field as u64;

        // safety: `addr` is a valid offset in the local APIC
        unsafe { read_volatile(addr as *const u32) }
    }

    pub fn write(&mut self, field: LocalAPICField, value: u32) {
        let addr = self.base_address.as_u64() + field as u64;

        // safety: `addr` is a valid offset in the local APIC
        unsafe { write_volatile(addr as *mut u32, value) }
    }

    pub fn id(&self) -> u32 {
        self.read(LocalAPICField::Id)
    }

    pub fn version(&self) -> u32 {
        self.read(LocalAPICField::Version)
    }

    /// signal the end of an interrupt
    /// should be called at the end of each apic related interrupt handler
    pub fn eoi(&self) {
        // cheat a bit and allow self to be immutable here. not sure if will be an issue but i want to avoid
        // deadlocks in interrupt handlers
        let addr = self.base_address.as_u64() + LocalAPICField::EOI as u64;

        // safety: `addr` is a valid offset in the local APIC
        unsafe { write_volatile(addr as *mut u32, 0) }
    }
}
