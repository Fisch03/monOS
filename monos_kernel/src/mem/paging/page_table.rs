use crate::mem::{Frame, PhysicalAddress};
use crate::utils::BitField;
use core::ops::{self, Range};

pub struct PageTableIndex(u16);
impl PageTableIndex {
    #[inline]
    pub const fn new(index: u16) -> Self {
        let index = index % 512;

        PageTableIndex(index)
    }

    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0 as u64
    }
}

#[derive(Clone)]
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    #[inline]
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        self.entries.iter()
    }
}

impl ops::Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl ops::IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl ops::Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;
    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[index.as_u64() as usize]
    }
}

impl ops::IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[index.as_u64() as usize]
    }
}

///   Page Table Entry
/// ┌──┬───────────────┐        
/// │ 0│    Present    │        
/// ├──┼───────────────┤        
/// │ 1│  Read/Write   │        
/// ├──┼───────────────┤        
/// │ 2│User/Supervisor│        
/// ├──┼───────────────┤        
/// │ 3│ Write-Through │        
/// ├──┼───────────────┤        
/// │ 4│ Cache Disable │        
/// ├──┼───────────────┤        
/// │ 5│   Accessed    │        
/// ├──┼───────────────┤        
/// │ 6│     Dirty     │        
/// ├──┼───────────────┼───────┐
/// │ 7│   Page Size   │P1/P4:0│
/// ├──┼───────────────┼───────┘
/// │ 8│    Global     │        
/// ├──┼───────────────┤        
/// │ 9│               │        
/// │  │   Available   │        
/// │11│               │        
/// ├──┼───────────────┤        
/// │12│               │        
/// │  │               │        
/// │  │    Address    │        
/// │  │               │        
/// │51│               │        
/// ├──┼───────────────┤        
/// │52│               │        
/// │  │   Available   │        
/// │62│               │        
/// ├──┼───────────────┤        
/// │63│  No Execute   │        
/// └──┴───────────────┘        
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const PRESENT: usize = 0;
    const WRITABLE: usize = 1;
    const USER_ACCESSIBLE: usize = 2;
    const WRITE_THROUGH: usize = 3;
    const CACHE_DISABLE: usize = 4;
    const ACCESSED: usize = 5;
    const DIRTY: usize = 6;
    const HUGE_PAGE: usize = 7;
    const GLOBAL: usize = 8;
    const ADDRESS: Range<usize> = 12..52;

    #[inline]
    const fn new() -> Self {
        PageTableEntry(0)
    }

    #[inline]
    pub fn is_present(&self) -> bool {
        self.0.get_bit(Self::PRESENT)
    }

    #[inline]
    pub fn is_huge(&self) -> bool {
        self.0.get_bit(Self::HUGE_PAGE)
    }

    #[inline]
    pub fn addr(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0.get_bits(Self::ADDRESS))
    }

    #[inline]
    pub fn frame(&self) -> Option<Frame> {
        if !self.is_present() || self.is_huge() {
            return None;
        } else {
            return Some(Frame::around(self.addr()));
        }
    }
}
