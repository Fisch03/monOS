use crate::mem::{Mapping, PhysicalAddress, VirtualAddress};
use crate::utils::BitField;
use core::ops::Range;

#[repr(C, packed)]
pub struct IOAPIC {
    ioregsel: u32,
    _reserved: [u32; 3],
    iowin: u32,
}

impl IOAPIC {
    pub unsafe fn new(address: PhysicalAddress) -> Mapping<Self> {
        Mapping::new(address, VirtualAddress::new(0xfee30000)).expect("failed to map IOAPIC")
    }

    #[inline]
    fn read_reg(&mut self, index: u32) -> u32 {
        self.ioregsel = index;
        self.iowin
    }

    #[inline]
    fn write_reg(&mut self, index: u32, data: u32) {
        self.ioregsel = index;
        self.iowin = data;
    }

    #[inline]
    fn read_reg64(&mut self, index: u32) -> u64 {
        let low = self.read_reg(index);
        let high = self.read_reg(index + 1);
        (high as u64) << 32 | low as u64
    }

    #[inline]
    fn write_reg64(&mut self, index: u32, data: u64) {
        self.write_reg(index, data as u32);
        self.write_reg(index + 1, (data >> 32) as u32);
    }

    #[inline]
    pub fn ioredtbl(&mut self, index: u32) -> IOREDTblEntry {
        let index = 0x10 + index * 2;
        IOREDTblEntry(self.read_reg64(index))
    }

    #[inline]
    pub fn set_ioredtbl(&mut self, index: u32, entry: IOREDTblEntry) {
        let index = 0x10 + index * 2;
        self.write_reg64(index, entry.0);
    }
}

#[repr(transparent)]
pub struct IOREDTblEntry(u64);
#[repr(u8)]
pub enum DeliveryMode {
    Fixed = 0b000,
    LowPriority = 0b001,
    SMI = 0b010,
    NMI = 0b100,
    INIT = 0b101,
    ExtINT = 0b111,
}

#[allow(dead_code)]
impl IOREDTblEntry {
    const VECTOR: Range<usize> = 0..8;
    const DELIVERY_MODE: Range<usize> = 8..11;
    const DESTINATION_MODE: usize = 11;
    // const DELIVERY_STATUS: usize = 12;
    const PIN_POLARITY: usize = 13;
    // const REMOTE_IRR: usize = 14;
    const TRIGGER_MODE: usize = 15;
    const MASK: usize = 16;
    const DESTINATION: Range<usize> = 56..64;

    #[inline]
    pub fn set_vector(&mut self, vector: u8) {
        self.0.set_bits(Self::VECTOR, vector.into());
    }

    #[inline]
    pub fn set_delivery_mode(&mut self, mode: DeliveryMode) {
        self.0.set_bits(Self::DELIVERY_MODE, mode as u64);
    }

    /// false = physical, true = logical
    #[inline]
    pub fn set_destination_mode(&mut self, mode: bool) {
        self.0.set_bit(Self::DESTINATION_MODE, mode);
    }

    /// false = active high, true = active low
    #[inline]
    pub fn set_pin_polarity(&mut self, polarity: bool) {
        self.0.set_bit(Self::PIN_POLARITY, polarity);
    }

    /// false = edge, true = level
    #[inline]
    pub fn set_trigger_mode(&mut self, mode: bool) {
        self.0.set_bit(Self::TRIGGER_MODE, mode);
    }

    #[inline]
    pub fn set_masked(&mut self, mask: bool) {
        self.0.set_bit(Self::MASK, mask);
    }

    #[inline]
    pub fn set_destination(&mut self, destination: u8) {
        self.0.set_bits(Self::DESTINATION, destination.into());
    }
}
