use super::frame::Frame;
use super::{PageSize, PageSize4K};
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

    #[inline]
    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        self.entries.iter_mut()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = PageTableEntry::new_empty();
        }
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

#[derive(Debug)]
pub enum PageTableFrameError {
    NotPresent,
    HugePage,
}

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
    #[inline]
    #[allow(dead_code)]
    const fn new_empty() -> Self {
        PageTableEntry(0)
    }

    #[inline]
    pub fn is_present(&self) -> bool {
        self.0.get_bit(PRESENT)
    }

    #[inline]
    pub fn is_huge(&self) -> bool {
        self.0.get_bit(HUGE_PAGE)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    #[inline]
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags(self.0.get_bits(0..12))
    }

    #[inline]
    pub unsafe fn set_frame<S: PageSize>(&mut self, frame: &Frame<S>) {
        self.set_addr(frame.start_address());
    }

    // safety: the flags must be valid.
    #[inline]
    pub unsafe fn set_flags(&mut self, flags: &PageTableFlags) {
        self.0.set_bits(0..12, flags.as_u64());
    }

    #[inline]
    pub fn addr(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.0.get_bits(ADDRESS) << 12)
    }
    #[inline]
    pub unsafe fn set_addr(&mut self, addr: PhysicalAddress) {
        self.0.set_bits(ADDRESS, addr.as_u64() >> 12);
    }

    #[inline]
    pub fn frame_4k(&self) -> Result<Frame<PageSize4K>, PageTableFrameError> {
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
            .field("writable", &self.0.get_bit(WRITABLE))
            .field("user_accessible", &self.0.get_bit(USER_ACCESSIBLE))
            .field("write_through", &self.0.get_bit(WRITE_THROUGH))
            .field("cache_disable", &self.0.get_bit(CACHE_DISABLE))
            .field("accessed", &self.0.get_bit(ACCESSED))
            .field("dirty", &self.0.get_bit(DIRTY))
            .field("huge_page", &self.0.get_bit(HUGE_PAGE))
            .field("global", &self.0.get_bit(GLOBAL))
            .field("address", &self.addr())
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageTableFlags(u64);
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageTableFlag(u64);

#[allow(dead_code)]
impl PageTableFlags {
    pub const PRESENT: PageTableFlag = PageTableFlag(1 << PRESENT);
    pub const WRITABLE: PageTableFlag = PageTableFlag(1 << WRITABLE);
    pub const USER_ACCESSIBLE: PageTableFlag = PageTableFlag(1 << USER_ACCESSIBLE);
    pub const WRITE_THROUGH: PageTableFlag = PageTableFlag(1 << WRITE_THROUGH);
    pub const CACHE_DISABLE: PageTableFlag = PageTableFlag(1 << CACHE_DISABLE);
    pub const ACCESSED: PageTableFlag = PageTableFlag(1 << ACCESSED);
    pub const DIRTY: PageTableFlag = PageTableFlag(1 << DIRTY);
    pub const HUGE_PAGE: PageTableFlag = PageTableFlag(1 << HUGE_PAGE);
    pub const GLOBAL: PageTableFlag = PageTableFlag(1 << GLOBAL);

    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn mask_parent(&self) -> Self {
        Self(self.0)
            & (PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE)
    }

    #[inline]
    pub fn contains(&self, other: PageTableFlag) -> bool {
        self.0 & other.0 == other.0
    }
}

impl fmt::Debug for PageTableFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageTableFlags")
            .field("present", &self.0.get_bit(PRESENT))
            .field("writable", &self.0.get_bit(WRITABLE))
            .field("user_accessible", &self.0.get_bit(USER_ACCESSIBLE))
            .field("write_through", &self.0.get_bit(WRITE_THROUGH))
            .field("cache_disable", &self.0.get_bit(CACHE_DISABLE))
            .field("accessed", &self.0.get_bit(ACCESSED))
            .field("dirty", &self.0.get_bit(DIRTY))
            .field("huge_page", &self.0.get_bit(HUGE_PAGE))
            .field("global", &self.0.get_bit(GLOBAL))
            .finish()
    }
}

impl PageTableFlag {
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl ops::BitOr for PageTableFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        PageTableFlags(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for PageTableFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl ops::BitAnd for PageTableFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        PageTableFlags(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for PageTableFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl ops::BitOr<PageTableFlag> for PageTableFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: PageTableFlag) -> Self {
        PageTableFlags(self.0 | rhs.as_u64())
    }
}

impl ops::BitOrAssign<PageTableFlag> for PageTableFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: PageTableFlag) {
        self.0 |= rhs.as_u64();
    }
}

impl ops::BitAnd<PageTableFlag> for PageTableFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: PageTableFlag) -> Self {
        PageTableFlags(self.0 & rhs.as_u64())
    }
}

impl ops::BitAndAssign<PageTableFlag> for PageTableFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: PageTableFlag) {
        self.0 &= rhs.as_u64();
    }
}

impl ops::BitOr for PageTableFlag {
    type Output = PageTableFlags;

    #[inline]
    fn bitor(self, rhs: Self) -> PageTableFlags {
        PageTableFlags(self.as_u64() | rhs.as_u64())
    }
}

impl ops::BitAnd for PageTableFlag {
    type Output = PageTableFlags;

    #[inline]
    fn bitand(self, rhs: Self) -> PageTableFlags {
        PageTableFlags(self.as_u64() & rhs.as_u64())
    }
}

impl ops::BitOrAssign for PageTableFlag {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl ops::BitAndAssign for PageTableFlag {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
