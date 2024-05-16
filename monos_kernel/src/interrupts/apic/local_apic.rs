use crate::mem::VirtualAddress;
use core::ptr::{read_volatile, write_volatile};

use crate::interrupts::InterruptIndex;

const APIC_ID: u32 = 0x20;
const APIC_VERSION: u32 = 0x30;
const ASK_PRIORITY: u32 = 0x80;
const ARBITRATION_PRIORITY: u32 = 0x90;
const PROCESSOR_PRIORITY: u32 = 0xA0;
const EOI: u32 = 0xB0;
const REMOTE_READ: u32 = 0xC0;
const LOGICAL_DESTINATION: u32 = 0xD0;
const DESTINATION_FORMAT: u32 = 0xE0;

const SPURIOUS_INTERRUPT_VECTOR: u32 = 0xF0;
const SPURIOS_INTERRUPT_ENABLE: u32 = 1 << 8;
const SPUROS_INTERRUPT_FCC: u32 = 1 << 9;

const IN_SERVICE: u32 = 0x100;
const TRIGGER_MODE: u32 = 0x180;
const INTERRUPT_REQUEST: u32 = 0x200;
const ERROR_STATUS: u32 = 0x280;
const INTERRUPT_COMMAND_LOW: u32 = 0x300;
const INTERRUPT_COMMAND_HIGH: u32 = 0x310;
const TIMER_LVT: u32 = 0x320;
const THERMAL_LVT: u32 = 0x330;
const PERFORMANCE_COUNTER_LVT: u32 = 0x340;
const LINT0_LVT: u32 = 0x350;
const LINT1_LVT: u32 = 0x360;
const ERROR_LVT: u32 = 0x370;
const TIMER_INITIAL_COUNT: u32 = 0x380;
const TIMER_CURRENT_COUNT: u32 = 0x390;
const TIMER_DIVIDE_CONFIG: u32 = 0x3E0;

pub struct LocalAPIC {
    base_address: VirtualAddress,
}

impl LocalAPIC {
    pub fn new(base_address: VirtualAddress) -> Self {
        let mut local_apic = Self { base_address };

        crate::dbg!(local_apic.id());
        crate::dbg!(local_apic.version());

        // unsafe {
        //     local_apic.write(
        //         SPURIOUS_INTERRUPT_VECTOR,
        //         InterruptIndex::SpuriousInterrupt.as_u32() | SPURIOS_INTERRUPT_ENABLE,
        //     );
        // }

        local_apic
    }

    fn read(&self, offset: u32) -> u32 {
        let addr = self.base_address.as_u64() + offset as u64;
        unsafe { read_volatile(addr as *const u32) }
    }

    // safety: `offset` is a valid offset in the local APIC
    unsafe fn write(&mut self, offset: u32, value: u32) {
        let addr = self.base_address.as_u64() + offset as u64;
        unsafe { write_volatile(addr as *mut u32, value) }
    }

    fn id(&self) -> u32 {
        self.read(APIC_ID)
    }

    fn version(&self) -> u32 {
        self.read(APIC_VERSION)
    }
}
