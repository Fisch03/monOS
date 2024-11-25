use crate::acpi::{tables, ACPI_ROOT};
use crate::mem::{Mapping, PhysicalAddress};
use crate::utils::BitField;
use spin::Lazy;

pub static HPET: Lazy<HPET> = Lazy::new(|| {
    let hpet = ACPI_ROOT
        .get()
        .expect("ACPI not initialized yet")
        .get_table::<tables::HPET>()
        .expect("no HPET table found");

    HPET::new(hpet.base_address())
});

#[derive(Debug)]
pub struct HPET {
    mapping: Mapping<HPETData>,
    freq: u64,
}

#[repr(C)]
struct HPETData {
    capabilities: HPETCapabilities,
    _padding1: u64,
    configuration: HPETConfiguration,
    _padding2: u64,
    interrupt_status: HPETInterruptStatus,
    _padding3: [u64; 25],
    counter: u64,
    _padding4: u64,
    //timers: [HPETTimer; 32],
}

impl HPET {
    fn new(base_address: PhysicalAddress) -> Self {
        let mut hpet: Mapping<HPETData> =
            unsafe { Mapping::new(base_address, 4096) }.expect("failed to map HPET");

        let freq = (10 as u64).pow(15) / hpet.capabilities.period();
        hpet.configuration.set_enabled(true);

        Self {
            mapping: hpet,
            freq,
        }
    }

    pub fn boot_time_ms(&self) -> u64 {
        self.mapping.counter * 1000 / self.freq
    }
}

impl core::fmt::Debug for HPETData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HPETData")
            .field("capabilities", &self.capabilities)
            .field("configuration", &self.configuration)
            .field("interrupt_status", &self.interrupt_status)
            .field("counter", &self.counter)
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct HPETCapabilities(u64);
impl HPETCapabilities {
    pub fn num_timers(&self) -> u32 {
        self.0.get_bits(8..13) as u32
    }

    pub fn vendor_id(&self) -> u32 {
        self.0.get_bits(16..32) as u32
    }

    pub fn period(&self) -> u64 {
        self.0.get_bits(32..64)
    }
}

impl core::fmt::Debug for HPETCapabilities {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HPETCapabilities")
            .field("num_timers", &self.num_timers())
            .field("vendor_id", &self.vendor_id())
            .field("period", &self.period())
            .finish()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct HPETConfiguration(u64);

impl HPETConfiguration {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.0.set_bit(0, enabled);
    }

    pub fn enabled(&self) -> bool {
        self.0.get_bit(0)
    }

    pub fn legacy_replacement(&self) -> bool {
        self.0.get_bit(1)
    }
}

impl core::fmt::Debug for HPETConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HPETConfiguration")
            .field("enable", &self.enabled())
            .field("legacy_replacement", &self.legacy_replacement())
            .finish()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HPETInterruptStatus(u64);
