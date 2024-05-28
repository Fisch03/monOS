use super::{PrivilegeLevel, SegmentSelector, TaskStateSegment};

use crate::mem::{DTPointer, VirtualAddress};
use crate::utils::BitField;

use core::mem::size_of;
use core::ops::Range;

#[derive(Clone, Copy)]
#[repr(transparent)]
struct GDTEntry(u64);
impl GDTEntry {
    fn new(value: u64) -> Self {
        Self(value)
    }

    const fn null() -> Self {
        Self(0)
    }
}

impl core::fmt::Debug for GDTEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    descriptors: [GDTEntry; 8],
    index: usize,
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        Self {
            descriptors: [GDTEntry::null(); 8],
            index: 1, // first descriptor is always null
        }
    }

    pub fn add_descriptor(&mut self, descriptor: SegmentDescriptor) -> SegmentSelector {
        match descriptor {
            SegmentDescriptor::User(value) => {
                let index = self.add_value(value);
                SegmentSelector::new(index as u16, descriptor.privilege_level())
            }
            SegmentDescriptor::System(lower, upper) => {
                let index = self.add_value(lower);
                self.add_value(upper);
                SegmentSelector::new(index as u16, descriptor.privilege_level())
            }
        }
    }

    fn add_value(&mut self, value: u64) -> usize {
        assert!(self.index < self.descriptors.len());

        let index = self.index;
        self.descriptors[index] = GDTEntry::new(value);
        self.index += 1;

        index
    }

    pub fn load(&'static self) {
        let ptr = DTPointer {
            base: VirtualAddress::new(self.descriptors.as_ptr() as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe {
            ptr.load_gdt();
        }
    }
}

/// ┌──┬───────────┐                       
/// │ 0│           │                       
/// │  │  Limit    │                       
/// │15│           │                       
/// ├──┼───────────┤                       
/// │16│           │                       
/// │  │   Base    │                       
/// │39│           │                       
/// ├──┼───────────┼─┬────────────────────┐
/// │40│           │0│      Accessed      │
/// │  │           ├─┼────────────────────┤
/// │  │           │1│  Readable/Writable │
/// │  │           ├─┼────────────────────┤
/// │  │           │2│Direction/Conforming│
/// │  │           ├─┼────────────────────┤
/// │  │Access Byte│3│     Executable     │
/// │  │           ├─┼────────────────────┤
/// │  │           │4│     System/User    │
/// │  │           ├─┼────────────────────┤
/// │  │           │5│      Privilege     │
/// │  │           │6│        Level       │
/// │  │           ├─┼────────────────────┤
/// │47│           │7│       Present      │
/// ├──┼───────────┼─┴────────────────────┘
/// │48│           │                       
/// │  │  Limit    │                       
/// │51│           │                       
/// ├──┼───────────┼─┬─────────────────┐   
/// │52│           │0│Reserved         │   
/// │  │           ├─┼─────────────────┤   
/// │  │           │1│Long mode flag   │   
/// │  │   Flags   ├─┼─────────────────┤   
/// │  │           │2│Size flag        │   
/// │  │           ├─┼─────────────────┤   
/// │55│           │3│Granularity flag │   
/// ├──┼───────────┼─┴─────────────────┘   
/// │56│           │                       
/// │  │   Base    │                       
/// │63│           │                       
/// └──┴───────────┘       
#[derive(Debug, Clone, Copy)]
pub enum SegmentDescriptor {
    User(u64),
    System(u64, u64),
}

#[allow(dead_code)]
impl SegmentDescriptor {
    const BASE_LOWER: Range<usize> = 16..40;
    const BASE_UPPER: Range<usize> = 56..64;

    const LIMIT_LOWER: Range<usize> = 0..16;
    const LIMIT_UPPER: Range<usize> = 48..52;

    const ACCESS_BYTE_BASE: usize = 40;
    const ACCESS_BYTE: Range<usize> = Self::ACCESS_BYTE_BASE..48;
    const ACCESS_BYTE_ACCESSED: usize = Self::ACCESS_BYTE_BASE + 0;
    const ACCESS_BYTE_WRITABLE: usize = Self::ACCESS_BYTE_BASE + 1;
    const ACCESS_BYTE_DIRECTION_CONFORMING: usize = Self::ACCESS_BYTE_BASE + 2;
    const ACCESS_BYTE_EXECUTABLE: usize = Self::ACCESS_BYTE_BASE + 3;
    const ACCESS_BYTE_USER: usize = Self::ACCESS_BYTE_BASE + 4;
    const ACCESS_BYTE_PRIVILEGE_LEVEL: Range<usize> =
        Self::ACCESS_BYTE_BASE + 5..Self::ACCESS_BYTE_BASE + 7;
    const ACCESS_BYTE_PRESENT: usize = Self::ACCESS_BYTE_BASE + 7;

    const FLAGS_BASE: usize = 52;
    const FLAGS: Range<usize> = Self::FLAGS_BASE..56;
    const FLAGS_LONG_MODE: usize = Self::FLAGS_BASE + 1;
    const FLAGS_SIZE: usize = Self::FLAGS_BASE + 2;
    const FLAGS_GRANULARITY: usize = Self::FLAGS_BASE + 3;

    fn base_bits() -> u64 {
        let mut bits: u64 = 0;

        bits.set_bits(Self::LIMIT_LOWER, 0xFFFF);
        bits.set_bits(Self::LIMIT_UPPER, 0xF);

        bits.set_bit(Self::ACCESS_BYTE_USER, true);
        bits.set_bit(Self::ACCESS_BYTE_PRESENT, true);
        bits.set_bit(Self::ACCESS_BYTE_WRITABLE, true);
        bits.set_bit(Self::ACCESS_BYTE_ACCESSED, true);

        bits.set_bit(Self::FLAGS_GRANULARITY, true);

        //TODO (?): Set Limit to max value

        bits
    }

    #[inline]
    pub fn kernel_code() -> Self {
        let mut bits = Self::base_bits();
        bits.set_bit(Self::ACCESS_BYTE_EXECUTABLE, true);
        bits.set_bit(Self::FLAGS_LONG_MODE, true);

        Self::User(bits)
    }

    #[inline]
    pub fn kernel_data() -> Self {
        let mut bits = Self::base_bits();
        bits.set_bit(Self::FLAGS_SIZE, true);

        Self::User(bits)
    }

    #[inline]
    pub fn user_code() -> Self {
        let mut bits = Self::base_bits();
        bits.set_bit(Self::ACCESS_BYTE_EXECUTABLE, true);
        bits.set_bit(Self::FLAGS_LONG_MODE, true);

        bits.set_bits(
            Self::ACCESS_BYTE_PRIVILEGE_LEVEL,
            PrivilegeLevel::Ring3 as u64,
        );

        Self::User(bits)
    }

    #[inline]
    pub fn user_data() -> Self {
        let mut bits = Self::base_bits();
        bits.set_bit(Self::FLAGS_SIZE, true);

        bits.set_bits(
            Self::ACCESS_BYTE_PRIVILEGE_LEVEL,
            PrivilegeLevel::Ring3 as u64,
        );

        Self::User(bits)
    }

    #[inline]
    pub fn tss(tss: &'static TaskStateSegment) -> Self {
        let tss = tss as *const _ as u64;

        let mut bits_lower: u64 = 0;
        bits_lower.set_bits(Self::BASE_LOWER, tss.get_bits(0..24));
        bits_lower.set_bits(Self::BASE_UPPER, tss.get_bits(24..32));

        bits_lower.set_bits(
            Self::LIMIT_LOWER,
            (size_of::<TaskStateSegment>() - 1) as u64,
        );

        bits_lower.set_bit(Self::ACCESS_BYTE_ACCESSED, true);
        bits_lower.set_bit(Self::ACCESS_BYTE_EXECUTABLE, true);
        bits_lower.set_bit(Self::ACCESS_BYTE_PRESENT, true);

        let mut bits_upper: u64 = 0;
        bits_upper.set_bits(0..32, tss.get_bits(32..64));

        Self::System(bits_lower, bits_upper)
    }

    pub fn privilege_level(&self) -> PrivilegeLevel {
        let value = match self {
            Self::User(value) => value,
            Self::System(value, _) => value,
        };

        let dpl = value.get_bits(Self::ACCESS_BYTE_PRIVILEGE_LEVEL);
        PrivilegeLevel::from_u16(dpl as u16)
    }
}
