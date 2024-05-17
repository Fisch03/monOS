use super::frame::{Frame, FrameSize4K};
use crate::mem::PhysicalAddress;
use crate::utils::BitField;
use core::fmt;
use core::ops::{self, Range};

#[derive(Debug)]
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

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    #[inline]
    #[allow(dead_code)]
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

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.entries.iter().filter(|entry| entry.is_present()))
            .finish()
    }
}

pub enum PageTableFrameError {
    NotPresent,
    HugePage,
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
#[derive(Clone)]
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
    #[allow(dead_code)]
    const fn new_empty() -> Self {
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
        PhysicalAddress::new(self.0.get_bits(Self::ADDRESS) << 12)
    }

    #[inline]
    pub fn frame(&self) -> Result<Frame<FrameSize4K>, PageTableFrameError> {
        if !self.is_present() {
            Err(PageTableFrameError::NotPresent)
        } else if self.is_huge() {
            Err(PageTableFrameError::HugePage)
        } else {
            Ok(Frame::around(self.addr()))
        }
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("present", &self.is_present())
            .field("writable", &self.0.get_bit(PageTableEntry::WRITABLE))
            .field(
                "user_accessible",
                &self.0.get_bit(PageTableEntry::USER_ACCESSIBLE),
            )
            .field(
                "write_through",
                &self.0.get_bit(PageTableEntry::WRITE_THROUGH),
            )
            .field(
                "cache_disable",
                &self.0.get_bit(PageTableEntry::CACHE_DISABLE),
            )
            .field("accessed", &self.0.get_bit(PageTableEntry::ACCESSED))
            .field("dirty", &self.0.get_bit(PageTableEntry::DIRTY))
            .field("huge_page", &self.0.get_bit(PageTableEntry::HUGE_PAGE))
            .field("global", &self.0.get_bit(PageTableEntry::GLOBAL))
            .field("address", &self.addr())
            .finish()
    }
}
