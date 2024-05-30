use super::RSDP;
use crate::mem::{Mapping, PhysicalAddress};
use core::{fmt, mem::size_of, slice, str};

#[derive(Debug)]
pub struct ACPIRoot {
    mapping: Mapping<SDTHeader>,
    revision: u8,
}

impl ACPIRoot {
    pub fn new(rsdp: Mapping<RSDP>) -> Self {
        let revision = rsdp.revision();
        let root_address_phys = if revision == 0 {
            rsdp.rsdt_address()
        } else {
            rsdp.xsdt_address()
        };

        let mut mapping: Mapping<SDTHeader> =
            // we don't know the size of the table yet, so we just map 10 pages to be safe
            unsafe { Mapping::new(root_address_phys, 4096 * 10) }.expect("failed to map root sdt");

        let signature = if revision == 0 {
            RootSDT::SIGNATURE
        } else {
            ExtendedSDT::SIGNATURE
        };

        if !mapping.validate(signature) {
            panic!("invalid root table");
        }

        let length = mapping.length();
        unsafe { mapping.extend(length as u64 - size_of::<SDTHeader>() as u64) };

        Self { mapping, revision }
    }

    fn tables_iter(&self) -> impl Iterator<Item = PhysicalAddress> {
        let tables_start = self.mapping.start_addr() + size_of::<SDTHeader>() as u64;
        let tables_end = self.mapping.start_addr() + self.mapping.length() as u64;

        let tables_len = tables_end - tables_start;
        let ptr_size = if self.revision == 0 { 4 } else { 8 };

        let bytes =
            unsafe { slice::from_raw_parts(tables_start.as_ptr::<u8>(), tables_len as usize) };

        bytes.chunks_exact(ptr_size).filter_map(|ptr| {
            let addr = u64::from_le_bytes(ptr.try_into().unwrap());
            PhysicalAddress::try_new(addr)
        })
    }

    pub fn get_table<T: SDT + fmt::Debug>(&self) -> Option<Mapping<T>> {
        self.tables_iter().find_map(|table_addr| {
            let table: Mapping<SDTHeader> =
                // go a bit overbard again to be safe
                unsafe { Mapping::new(table_addr, 4096 * 10) }.expect("failed to map table");

            if !table.validate(T::SIGNATURE) {
                return None;
            }

            let mut table = unsafe { table.cast::<T>() };
            unsafe { table.extend(table.header().length() as u64 - table.size()) };

            Some(table)
        })
    }
}

#[repr(C, packed)]
pub struct SDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

impl SDTHeader {
    pub fn validate(&self, signature: &[u8; 4]) -> bool {
        if &self.signature != signature {
            return false;
        }

        if str::from_utf8(&self.oem_id).is_err() {
            return false;
        }

        if str::from_utf8(&self.oem_table_id).is_err() {
            return false;
        }

        let mut sum = 0u8;
        for i in 0..self.length as usize {
            // safety: we only read data and don't do anything critical with it
            sum = sum.wrapping_add(unsafe { *(self as *const Self as *const u8).add(i) });
        }

        if sum != 0 {
            return false;
        }

        return true;
    }

    #[inline]
    pub fn length(&self) -> u32 {
        self.length
    }
}

impl fmt::Debug for SDTHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let length = self.length;
        let oem_revision = self.oem_revision;
        let creator_id = self.creator_id;
        let creator_revision = self.creator_revision;

        f.debug_struct("SDTHeader")
            .field("signature", &str::from_utf8(&self.signature).unwrap())
            .field("length", &length)
            .field("revision", &self.revision)
            .field("checksum", &self.checksum)
            .field("oem_id", &str::from_utf8(&self.oem_id).unwrap())
            .field("oem_table_id", &str::from_utf8(&self.oem_table_id).unwrap())
            .field("oem_revision", &oem_revision)
            .field("creator_id", &creator_id)
            .field("creator_revision", &creator_revision)
            .finish()
    }
}

pub trait SDT {
    const SIGNATURE: &'static [u8; 4];

    fn header(&self) -> &SDTHeader;
}

#[repr(C, packed)]
struct RootSDT {
    header: SDTHeader,
}
impl SDT for RootSDT {
    const SIGNATURE: &'static [u8; 4] = b"RSDT";
    fn header(&self) -> &SDTHeader {
        &self.header
    }
}
impl fmt::Debug for RootSDT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootSDT")
            .field("header", &self.header)
            .finish()
    }
}

#[repr(C, packed)]
struct ExtendedSDT {
    header: SDTHeader,
}
impl SDT for ExtendedSDT {
    const SIGNATURE: &'static [u8; 4] = b"XSDT";
    fn header(&self) -> &SDTHeader {
        &self.header
    }
}
impl fmt::Debug for ExtendedSDT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtendedSDT")
            .field("header", &self.header)
            .finish()
    }
}
