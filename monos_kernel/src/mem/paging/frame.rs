use super::{PageSize, PageSize1G, PageSize2M, PageSize4K};
use crate::mem::PhysicalAddress;
use core::marker::PhantomData;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Frame<Size: PageSize = PageSize4K> {
    start: PhysicalAddress,
    size: PhantomData<Size>,
}

#[derive(Debug)]
pub enum MappedFrame {
    Size4K(Frame<PageSize4K>),
    Size2M(Frame<PageSize2M>),
    Size1G(Frame<PageSize1G>),
}

impl<S: PageSize> Frame<S> {
    #[inline]
    pub fn new(start: PhysicalAddress) -> Option<Self> {
        if !start.is_aligned(S::SIZE) {
            return None;
        }

        // safety: we just checked that the address is aligned to the start of the frame.
        unsafe { Some(Self::new_unchecked(start)) }
    }

    // safety: the address must be aligned to the start of the frame.
    #[inline]
    pub const unsafe fn new_unchecked(start: PhysicalAddress) -> Self {
        Frame {
            start,
            size: PhantomData,
        }
    }

    #[inline]
    pub fn around(addr: PhysicalAddress) -> Self {
        Frame {
            start: addr.align(S::SIZE),
            size: PhantomData,
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn start_address(&self) -> PhysicalAddress {
        self.start
    }

    #[inline]
    #[allow(dead_code)]
    pub fn end_address(&self) -> PhysicalAddress {
        self.start + S::SIZE
    }

    #[inline]
    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        S::SIZE
    }
}

impl MappedFrame {
    #[inline]
    #[allow(dead_code)]
    pub fn start_address(&self) -> PhysicalAddress {
        match self {
            MappedFrame::Size4K(frame) => frame.start_address(),
            MappedFrame::Size2M(frame) => frame.start_address(),
            MappedFrame::Size1G(frame) => frame.start_address(),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        match self {
            MappedFrame::Size4K(frame) => frame.size(),
            MappedFrame::Size2M(frame) => frame.size(),
            MappedFrame::Size1G(frame) => frame.size(),
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn end_address(&self) -> PhysicalAddress {
        self.start_address() + self.size()
    }
}
