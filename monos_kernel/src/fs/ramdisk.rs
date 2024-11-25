use super::{Read, Seek, Write};
use crate::mem::VirtualAddress;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct RamDisk {
    start: VirtualAddress,
    size: usize,
    pos: AtomicUsize,
}

impl RamDisk {
    // safety: address and start must point to a valid (mapped) memory region and there cannot be
    // any aliasing
    pub unsafe fn new(start: VirtualAddress, size: usize) -> Self {
        Self {
            start,
            size,
            pos: AtomicUsize::new(0),
        }
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.start.as_mut_ptr(), self.size) }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.start.as_mut_ptr(), self.size) }
    }

    /// safety: the fs implementation must guarantee that no aliasing occurs
    pub unsafe fn clone(&self) -> Self {
        Self {
            start: self.start,
            size: self.size,
            pos: AtomicUsize::new(0),
        }
    }
}

impl Read for RamDisk {
    fn read(&self, buf: &mut [u8]) -> usize {
        let pos = self.pos.fetch_add(buf.len(), Ordering::Relaxed);
        let len = core::cmp::min(buf.len(), self.size - pos);
        buf.copy_from_slice(&self.as_slice()[pos..pos + len]);
        len
    }
}

impl Write for RamDisk {
    fn write(&mut self, buf: &[u8]) -> usize {
        let pos = self.pos.fetch_add(buf.len(), Ordering::Relaxed);
        let len = core::cmp::min(buf.len(), self.size - pos);
        self.as_mut_slice()[pos..pos + len].copy_from_slice(buf);
        len
    }
}

impl Seek for RamDisk {
    fn set_pos(&self, pos: usize) {
        self.pos.store(pos, Ordering::Relaxed);
    }

    fn get_pos(&self) -> usize {
        self.pos.load(Ordering::Relaxed)
    }

    fn max_pos(&self) -> usize {
        self.size
    }
}
