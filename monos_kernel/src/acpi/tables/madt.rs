use crate::acpi::sdt::{SDTHeader, SDT};
use crate::{
    interrupts::apic,
    mem::{Mapping, PhysicalAddress},
};
use core::fmt;

/// Multiple APIC Description Table
#[repr(C, packed)]
pub struct MADT {
    header: SDTHeader,
    local_interrupt_controller_address: u32,
    flags: u32,
    // rest of the table is variable length
}
impl SDT for MADT {
    const SIGNATURE: &'static [u8; 4] = b"APIC";
    fn header(&self) -> &SDTHeader {
        &self.header
    }
}
impl fmt::Debug for MADT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lic_address = self.local_interrupt_controller_address;
        let flags = self.flags;

        f.debug_struct("MADT")
            .field("header", &self.header)
            .field("local_interrupt_controller_address", &lic_address)
            .field("flags", &flags)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum MADTEntryType {
    ProcessorLocalAPIC = 0,
    IOAPIC = 1,
    InterruptSourceOverride = 2,
    NMISource = 3,
    LocalAPICNMI = 4,
    LocalAPICAddressOverride = 5,
    IOSAPIC = 6,
    LocalSAPIC = 7,
    PlatformInterruptSources = 8,
    ProcessorLocalx2APIC = 9,
    Localx2APICNMI = 0xA,
    GICC = 0xB,
    GICD = 0xC,
    GICMSIFrame = 0xD,
    GICR = 0xE,
    GICITS = 0xF,
    MultiprocessorWakeUp = 0x10,
}

pub trait MADTEntry {
    const ENTRY_TYPE: MADTEntryType;
}

struct MADTEntryIterator {
    current: *const u8,
    end: *const u8,
}
impl Iterator for MADTEntryIterator {
    type Item = (MADTEntryType, &'static [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let entry_type = unsafe { *(self.current as *const MADTEntryType) };
        let entry_length = unsafe { *((self.current.add(1)) as *const u8) };
        let entry = unsafe { core::slice::from_raw_parts(self.current, entry_length as usize) };

        let full_length = entry_length as usize;
        self.current = unsafe { self.current.add(full_length) };
        Some((entry_type, entry))
    }
}

impl MADT {
    fn iter_entries(&self) -> MADTEntryIterator {
        let start = unsafe { (self as *const MADT).add(1) } as *const u8;
        let end = unsafe { (self as *const MADT as *const u8).add(self.header.length() as usize) };

        MADTEntryIterator {
            current: start,
            end,
        }
    }

    pub fn get_entries<'a, T: MADTEntry + 'a>(&'a self) -> impl Iterator<Item = &'a T> {
        self.iter_entries().filter_map(|(entry_type, entry)| {
            if entry_type == T::ENTRY_TYPE {
                Some(unsafe { &*(entry as *const [u8] as *const T) })
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct ProcessorLocalAPIC {
    entry_type: u8,
    length: u8,

    processor_uid: u8,
    apic_id: u8,
    flags: u32,
}
impl ProcessorLocalAPIC {
    #[inline]
    pub fn processor_uid(&self) -> u8 {
        self.processor_uid
    }
    #[inline]
    pub fn apic_id(&self) -> u8 {
        self.apic_id
    }
}
impl MADTEntry for ProcessorLocalAPIC {
    const ENTRY_TYPE: MADTEntryType = MADTEntryType::ProcessorLocalAPIC;
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct IOAPIC {
    entry_type: u8,
    length: u8,

    ioapic_id: u8,
    reserved: u8,
    ioapic_address: u32,
    global_system_interrupt_base: u32,
}
impl IOAPIC {
    #[inline]
    pub fn get_ioapic(&self) -> Mapping<apic::IOAPIC> {
        // safety: since the address comes directly from the ACPI table, it is guaranteed to be valid.
        unsafe { apic::IOAPIC::new(PhysicalAddress::new(self.ioapic_address as u64)) }
    }
}
impl MADTEntry for IOAPIC {
    const ENTRY_TYPE: MADTEntryType = MADTEntryType::IOAPIC;
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct InterruptSourceOverride {
    entry_type: u8,
    length: u8,

    bus: u8,
    source: u8,
    global_system_interrupt: u32,
    flags: u16,
}
impl InterruptSourceOverride {
    #[inline]
    pub fn source(&self) -> u8 {
        self.source
    }
    #[inline]
    pub fn global_system_interrupt(&self) -> u32 {
        self.global_system_interrupt
    }
}
impl MADTEntry for InterruptSourceOverride {
    const ENTRY_TYPE: MADTEntryType = MADTEntryType::InterruptSourceOverride;
}
