use crate::mem::VirtualAddress;

pub struct RamDisk {
    data: &'static mut [u8],
}

impl RamDisk {
    // safety: address and start must point to a valid (mapped) memory region and there cannot be
    // any aliasing
    pub unsafe fn new(start: VirtualAddress, size: usize) -> Self {
        let data = core::slice::from_raw_parts_mut(start.as_mut_ptr(), size);
        Self { data }
    }

    pub fn read(&self, offset: usize, buf: &mut [u8]) {
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
    }

    pub fn write(&mut self, offset: usize, buf: &[u8]) {
        self.data[offset..offset + buf.len()].copy_from_slice(buf);
    }
}
