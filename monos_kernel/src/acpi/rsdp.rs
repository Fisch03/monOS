use crate::mem::{self, PhysicalAddress, VirtualAddress};
use core::str;

#[derive(Debug)]
#[repr(C, packed)]
pub struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: [u8; 4],

    // ACPI v2 fields
    length: u32,
    xsdt_address: PhysicalAddress,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[derive(Debug)]
pub enum RSDPError {
    InvalidChecksum,
    InvalidOEMId,
    InvalidSignature,
}

const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";
impl RSDP {
    pub fn new(rsdp_addr: PhysicalAddress) -> Result<VirtualAddress, RSDPError> {
        // this should be fairly safe since we validate the checksum of the rsdp
        let frame = mem::Frame::around(rsdp_addr);
        let page = mem::Page::around(VirtualAddress::new(0xfee10000));

        use mem::PageTableFlags;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe { mem::map_to(&page, &frame, flags) }.expect("failed to map apic memory");

        let rsdp_offset = rsdp_addr.offset_in_page();
        let rsdp_addr = page.start_address() + rsdp_offset;

        // safety: we will validate the (probably) rsdp before returning it
        let rsdp = unsafe { &*(rsdp_addr.as_ptr() as *const RSDP) };
        rsdp.validate()?;

        Ok(rsdp_addr)
    }

    fn validate(&self) -> Result<(), RSDPError> {
        if &self.signature != &RSDP_SIGNATURE {
            return Err(RSDPError::InvalidSignature);
        }

        if str::from_utf8(&self.oem_id).is_err() {
            return Err(RSDPError::InvalidOEMId);
        }

        let len = if self.is_version_2() {
            self.length as usize
        } else {
            20
        };

        let mut sum = 0u8;
        for i in 0..len {
            // safety: we only read data and don't do anything critical with it
            sum = sum.wrapping_add(unsafe { *(self as *const RSDP as *const u8).add(i) });
        }

        if sum != 0 {
            return Err(RSDPError::InvalidChecksum);
        }

        Ok(())
    }

    #[inline]
    fn is_version_2(&self) -> bool {
        self.revision > 0
    }
}
