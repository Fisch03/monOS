use crate::acpi::sdt::{SDTHeader, SDT};
use crate::mem::PhysicalAddress;

#[repr(C, packed)]
pub struct HPET {
    header: SDTHeader,
    hardware_rev_id: u8,
    comparator_count_and_offset: u8,
    pci_vendor_id: u16,
    address: HPETAddressStructure,
    hpet_number: u8,
    minimum_tick: u16,
    page_protection: u8,
}

impl SDT for HPET {
    const SIGNATURE: &'static [u8; 4] = b"HPET";
    fn header(&self) -> &SDTHeader {
        &self.header
    }
}

impl HPET {
    pub fn base_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.address.address)
    }
}

impl core::fmt::Debug for HPET {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hardware_rev_id = self.hardware_rev_id;
        let comparator_count_and_offset = self.comparator_count_and_offset;
        let pci_vendor_id = self.pci_vendor_id;
        let hpet_number = self.hpet_number;
        let minimum_tick = self.minimum_tick;
        let page_protection = self.page_protection;
        f.debug_struct("HPET")
            .field("header", &self.header)
            .field("hardware_rev_id", &hardware_rev_id)
            .field("comparator_count_and_offset", &comparator_count_and_offset)
            .field("pci_vendor_id", &pci_vendor_id)
            .field("address", &self.address)
            .field("hpet_number", &hpet_number)
            .field("minimum_tick", &minimum_tick)
            .field("page_protection", &page_protection)
            .finish()
    }
}

#[repr(C, packed)]
struct HPETAddressStructure {
    address_space_id: u8,
    register_bit_width: u8,
    register_bit_offset: u8,
    reserved: u8,
    address: u64,
}

impl core::fmt::Debug for HPETAddressStructure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let address_space_id = self.address_space_id;
        let register_bit_width = self.register_bit_width;
        let register_bit_offset = self.register_bit_offset;
        let address = self.address;
        f.debug_struct("HPETAddressStructure")
            .field("address_space_id", &address_space_id)
            .field("register_bit_width", &register_bit_width)
            .field("register_bit_offset", &register_bit_offset)
            .field("address", &PhysicalAddress::new(address))
            .finish()
    }
}
