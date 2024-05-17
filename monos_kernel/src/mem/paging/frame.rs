use crate::mem::PhysicalAddress;
use core::marker::PhantomData;

pub trait FrameSize: Copy {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy)]
pub struct FrameSize4K;
impl FrameSize for FrameSize4K {
    const SIZE: u64 = 4096;
}

#[derive(Debug, Clone, Copy)]
pub struct FrameSize2M;
impl FrameSize for FrameSize2M {
    const SIZE: u64 = 4096 * 512;
}

#[derive(Debug, Clone, Copy)]
pub struct FrameSize1G;
impl FrameSize for FrameSize1G {
    const SIZE: u64 = 4096 * 512 * 512;
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Frame<Size: FrameSize> {
    start: PhysicalAddress,
    size: PhantomData<Size>,
}

#[derive(Debug)]
pub enum MappedFrame {
    Size4K(Frame<FrameSize4K>),
    Size2M(Frame<FrameSize2M>),
    Size1G(Frame<FrameSize1G>),
}

impl<S: FrameSize> Frame<S> {
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
