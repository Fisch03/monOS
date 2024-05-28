use crate::mem::{Mapping, PhysicalAddress, VirtualAddress};
use core::{fmt, mem, str};

#[repr(C, packed)]
pub struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,

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
    pub fn new(rsdp_addr: PhysicalAddress) -> Result<Mapping<Self>, RSDPError> {
        // this should be fairly safe since we validate the checksum of the rsdp
        let mapping: Mapping<Self> =
            unsafe { Mapping::new(rsdp_addr, mem::size_of::<RSDP>()) }.expect("failed to map rsdp");

        mapping.validate()?;

        Ok(mapping)
    }

    fn validate(&self) -> Result<(), RSDPError> {
        if &self.signature != &RSDP_SIGNATURE {
            return Err(RSDPError::InvalidSignature);
        }

        if str::from_utf8(&self.oem_id).is_err() {
            return Err(RSDPError::InvalidOEMId);
        }

        let len = if self.is_version_1() {
            20
        } else {
            self.length as usize
        };

        let mut sum = 0u8;
        for i in 0..len {
            // safety: we only read data and don't do anything critical with it
            sum = sum.wrapping_add(unsafe { *(self as *const Self as *const u8).add(i) });
        }

        if sum != 0 {
            return Err(RSDPError::InvalidChecksum);
        }

        Ok(())
    }

    #[inline]
    fn is_version_1(&self) -> bool {
        self.revision == 0
    }

    #[inline]
    pub fn revision(&self) -> u8 {
        self.revision
    }

    #[inline]
    pub fn rsdt_address(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.rsdt_address as u64)
    }

    #[inline]
    pub fn xsdt_address(&self) -> PhysicalAddress {
        assert!(self.revision > 0, "rsdp is version 1");
        self.xsdt_address
    }
}

impl fmt::Debug for RSDP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let length = self.length;

        f.debug_struct("RSDP")
            .field("signature", &str::from_utf8(&self.signature).unwrap())
            .field("checksum", &self.checksum)
            .field("oem_id", &str::from_utf8(&self.oem_id).unwrap())
            .field("revision", &self.revision)
            .field("rsdt_address", &self.rsdt_address())
            .field("length", &length)
            .field("xsdt_address", &self.xsdt_address())
            .field("extended_checksum", &self.extended_checksum)
            .finish()
    }
}
