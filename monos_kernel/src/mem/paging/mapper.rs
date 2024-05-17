use super::{
    active_level_4_table,
    frame::{Frame, FrameSize, MappedFrame},
    page_table::{PageTable, PageTableFrameError},
};
use crate::mem::{PhysicalAddress, VirtualAddress};

#[derive(Debug)]
pub struct AddressMapping {
    frame: MappedFrame,
    offset: u64,
}
impl AddressMapping {
    pub fn to_physical(&self) -> PhysicalAddress {
        self.frame.start_address() + self.offset
    }
}

#[derive(Debug)]
pub enum TranslateError {
    NotMapped,
    InvalidAddress,
}

#[derive(Debug)]
pub struct Mapper<'pt> {
    l4: &'pt mut PageTable,
    offset: VirtualAddress,
}

impl Mapper<'_> {
    /// safety: the physical memory offset must be valid and the page tables need to be set up correctly.
    pub unsafe fn new(physical_mem_offset: VirtualAddress) -> Self {
        let l4 = unsafe { active_level_4_table(physical_mem_offset) };
        Self {
            l4,
            offset: physical_mem_offset,
        }
    }

    /// convenience function for translating a virtual address to a physical address.
    pub fn translate_addr(&self, addr: VirtualAddress) -> Result<PhysicalAddress, TranslateError> {
        match self.translate(addr) {
            Ok(mapping) => Ok(mapping.to_physical()),
            Err(err) => Err(err),
        }
    }

    pub fn translate(&self, addr: VirtualAddress) -> Result<AddressMapping, TranslateError> {
        let l4 = &self.l4;

        let l4_entry = &l4[addr.p4_index()];
        let l3_frame = match l4_entry.frame() {
            Ok(frame) => frame,
            Err(PageTableFrameError::NotPresent) => return Err(TranslateError::NotMapped),
            Err(PageTableFrameError::HugePage) => panic!("l4 entry is marked as huge page"),
        };

        let l3 = self.frame_to_table(&l3_frame);
        let l3_entry = &l3[addr.p3_index()];
        let l2_frame = match l3_entry.frame() {
            Ok(frame) => frame,
            Err(PageTableFrameError::NotPresent) => return Err(TranslateError::NotMapped),
            Err(PageTableFrameError::HugePage) => {
                let frame = Frame::around(l3_entry.addr());
                let offset = addr.as_u64() & 0o_777_777_7777;
                return Ok(AddressMapping {
                    frame: MappedFrame::Size1G(frame),
                    offset,
                });
            }
        };

        let l2 = self.frame_to_table(&l2_frame);
        let l2_entry = &l2[addr.p2_index()];
        let l1_frame = match l2_entry.frame() {
            Ok(frame) => frame,
            Err(PageTableFrameError::NotPresent) => return Err(TranslateError::NotMapped),
            Err(PageTableFrameError::HugePage) => {
                let frame = Frame::around(l2_entry.addr());
                let offset = addr.as_u64() & 0o_777_777;
                return Ok(AddressMapping {
                    frame: MappedFrame::Size2M(frame),
                    offset,
                });
            }
        };

        let l1 = self.frame_to_table(&l1_frame);
        let l1_entry = &l1[addr.p1_index()];
        if !l1_entry.is_present() {
            return Err(TranslateError::NotMapped);
        }

        if let Some(frame) = Frame::new(l1_entry.addr()) {
            let offset = u64::from(addr.page_offset());
            Ok(AddressMapping {
                frame: MappedFrame::Size4K(frame),
                offset,
            })
        } else {
            Err(TranslateError::InvalidAddress)
        }
    }

    fn frame_to_table<'a, S>(&self, frame: &'a Frame<S>) -> &'a mut PageTable
    where
        S: FrameSize,
    {
        let virt = self.offset + frame.start_address().as_u64();
        let ptr: *mut PageTable = virt.as_mut_ptr();
        unsafe { &mut *ptr }
    }
}
