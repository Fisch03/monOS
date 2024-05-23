use super::PageSize4K;
use super::{page_table::PageTableIndex, PageSize};
use crate::mem::VirtualAddress;
use core::arch::asm;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page<Size: PageSize = PageSize4K> {
    start: VirtualAddress,
    size: PhantomData<Size>,
}

impl<S: PageSize> Page<S> {
    #[inline]
    #[allow(dead_code)]
    pub fn new(start: VirtualAddress) -> Option<Self> {
        if !start.is_aligned(S::SIZE) {
            return None;
        }

        // safety: we just checked that the address is aligned to the start of the frame.
        unsafe { Some(Self::new_unchecked(start)) }
    }

    // safety: the address must be aligned to the start of the frame.
    #[inline]
    #[allow(dead_code)]
    pub const unsafe fn new_unchecked(start: VirtualAddress) -> Self {
        Page {
            start,
            size: PhantomData,
        }
    }

    #[inline]
    pub fn flush(&self) {
        unsafe {
            asm!("invlpg [{}]", in(reg) self.start_address().as_u64(), options(nostack, preserves_flags));
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn around(addr: VirtualAddress) -> Self {
        Page {
            start: addr.align(S::SIZE),
            size: PhantomData,
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        Self {
            start: self.start + S::SIZE,
            size: PhantomData,
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn start_address(&self) -> VirtualAddress {
        self.start
    }

    #[inline]
    #[allow(dead_code)]
    pub fn end_address(&self) -> VirtualAddress {
        self.start + S::SIZE
    }

    #[inline]
    #[allow(dead_code)]
    pub fn p4_index(&self) -> PageTableIndex {
        self.start.p4_index()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn p3_index(&self) -> PageTableIndex {
        self.start.p3_index()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn p2_index(&self) -> PageTableIndex {
        self.start.p2_index()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn p1_index(&self) -> PageTableIndex {
        self.start.p1_index()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        S::SIZE
    }
}
