use super::{
    alloc_frame,
    frame::{Frame, MappedFrame},
    page::Page,
    page_table::{PageTable, PageTableEntry, PageTableFlags, PageTableFrameError},
    PageSize, PageSize1G, PageSize2M, PageSize4K,
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

pub trait MapTo<S: PageSize> {
    unsafe fn map_to(
        &mut self,
        page: &Page<S>,
        frame: &Frame<S>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError>;

    fn unmap(&mut self, page: &Page<S>) -> Result<(), UnmapError>;
}

#[derive(Debug)]
pub enum MapToError {
    ParentHugePage,
    AlreadyMapped,
    OutOfMemory,
}

#[derive(Debug)]
pub enum UnmapError {
    ParentHugePage,
    NotMapped,
}
impl From<PageTableFrameError> for UnmapError {
    fn from(err: PageTableFrameError) -> Self {
        match err {
            PageTableFrameError::NotPresent => Self::NotMapped,
            PageTableFrameError::HugePage => Self::ParentHugePage,
        }
    }
}

#[derive(Debug)]
pub struct Mapper<'pt> {
    l4: &'pt mut PageTable,
    manager: PageManager,
}

impl<'pt> Mapper<'pt> {
    /// safety: the physical memory offset must be valid and the page tables need to be set up correctly.
    #[inline]
    pub unsafe fn new(physical_mem_offset: VirtualAddress, l4: &'pt mut PageTable) -> Self {
        let manager = unsafe { PageManager::new(physical_mem_offset) };
        Self { l4, manager }
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
        let l3 = match self.manager.entry_to_table(l4_entry) {
            Ok(table) => table,
            Err(PageTableFrameError::NotPresent) => return Err(TranslateError::NotMapped),
            Err(PageTableFrameError::HugePage) => panic!("l4 entry is marked as huge page"),
        };

        let l3_entry = &l3[addr.p3_index()];
        let l2 = match self.manager.entry_to_table(l3_entry) {
            Ok(table) => table,
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

        let l2_entry = &l2[addr.p2_index()];
        let l1 = match self.manager.entry_to_table(l2_entry) {
            Ok(table) => table,
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
}

/// safety: the flags must be valid. the new mapping must not cause any undefined behavior (overlapping physical addresses, etc).
impl MapTo<PageSize4K> for Mapper<'_> {
    unsafe fn map_to(
        &mut self,
        page: &Page<PageSize4K>,
        frame: &Frame<PageSize4K>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError> {
        let parent_flags = flags.mask_parent();

        let l3 = self
            .manager
            .get_or_create_table(&mut self.l4[page.p4_index()], parent_flags)?;

        let l2 = self
            .manager
            .get_or_create_table(&mut l3[page.p3_index()], parent_flags)?;

        let l1 = self
            .manager
            .get_or_create_table(&mut l2[page.p2_index()], parent_flags)?;

        let entry = &mut l1[page.p1_index()];

        // TODO: check if it's fine to just overwrite the frame and flags.
        // since we allocate new frames ourselves and we don't allow overlapping mappings, this should be fine.
        // if entry.is_present() {
        //     return Err(MapToError::AlreadyMapped);
        // }

        entry.set_frame(frame);
        entry.set_flags(&flags);

        page.flush();

        Ok(())
    }

    fn unmap(&mut self, page: &Page<PageSize4K>) -> Result<(), UnmapError> {
        let l3 = self.manager.entry_to_table(&mut self.l4[page.p4_index()])?;
        let l2 = self.manager.entry_to_table(&mut l3[page.p3_index()])?;
        let l1 = self.manager.entry_to_table(&mut l2[page.p2_index()])?;

        let entry = &mut l1[page.p1_index()];
        entry.frame()?;

        entry.clear();
        page.flush();

        Ok(())
    }
}

impl MapTo<PageSize2M> for Mapper<'_> {
    unsafe fn map_to(
        &mut self,
        page: &Page<PageSize2M>,
        frame: &Frame<PageSize2M>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError> {
        let parent_flags = flags.mask_parent();

        let l3 = self
            .manager
            .get_or_create_table(&mut self.l4[page.p4_index()], parent_flags)?;
        let l2 = self
            .manager
            .get_or_create_table(&mut l3[page.p3_index()], parent_flags)?;

        let entry = &mut l2[page.p2_index()];

        // TODO: check if it's fine to just overwrite the frame and flags.
        // since we allocate new frames ourselves and we don't allow overlapping mappings, this should be fine.
        // if entry.is_present() {
        //     return Err(MapToError::AlreadyMapped);
        // }

        entry.set_frame(frame);
        entry.set_flags(&(flags | PageTableFlags::HUGE_PAGE));

        page.flush();

        Ok(())
    }

    fn unmap(&mut self, page: &Page<PageSize2M>) -> Result<(), UnmapError> {
        let l3 = self.manager.entry_to_table(&mut self.l4[page.p4_index()])?;
        let l2 = self.manager.entry_to_table(&mut l3[page.p3_index()])?;

        let entry = &mut l2[page.p2_index()];
        entry.frame()?;

        entry.clear();
        page.flush();

        Ok(())
    }
}

impl MapTo<PageSize1G> for Mapper<'_> {
    unsafe fn map_to(
        &mut self,
        page: &Page<PageSize1G>,
        frame: &Frame<PageSize1G>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError> {
        let parent_flags = flags.mask_parent();

        let l3 = self
            .manager
            .get_or_create_table(&mut self.l4[page.p4_index()], parent_flags)?;

        let entry = &mut l3[page.p3_index()];

        // TODO: check if it's fine to just overwrite the frame and flags.
        // since we allocate new frames ourselves and we don't allow overlapping mappings, this should be fine.
        // if entry.is_present() {
        //     return Err(MapToError::AlreadyMapped);
        // }

        entry.set_frame(frame);
        entry.set_flags(&(flags | PageTableFlags::HUGE_PAGE));

        page.flush();

        Ok(())
    }

    fn unmap(&mut self, page: &Page<PageSize1G>) -> Result<(), UnmapError> {
        let l3 = self.manager.entry_to_table(&mut self.l4[page.p4_index()])?;

        let entry = &mut l3[page.p3_index()];
        entry.frame()?;

        entry.clear();
        page.flush();

        Ok(())
    }
}

#[derive(Debug)]
pub struct PageManager {
    offset: VirtualAddress,
}

impl PageManager {
    /// safety: the offset must be valid.
    #[inline]
    pub unsafe fn new(offset: VirtualAddress) -> Self {
        Self { offset }
    }

    fn entry_to_table<'a>(
        &self,
        entry: &'a PageTableEntry,
    ) -> Result<&'a mut PageTable, PageTableFrameError> {
        let frame = entry.frame()?;
        let virt = self.offset + frame.start_address().as_u64();
        let ptr: *mut PageTable = virt.as_mut_ptr();
        Ok(unsafe { &mut *ptr })
    }

    /// safety: the flags must be valid.
    unsafe fn get_or_create_table<'a>(
        &self,
        entry: &'a mut PageTableEntry,
        flags: PageTableFlags,
    ) -> Result<&'a mut PageTable, MapToError> {
        let new_frame = !entry.is_present();
        if new_frame {
            if let Some(frame) = alloc_frame() {
                entry.set_frame(&frame);
                entry.set_flags(&flags);
            } else {
                return Err(MapToError::OutOfMemory);
            }
        } else {
            let entry_flags = entry.flags();
            if entry_flags != flags {
                // safety: the caller guarantees that the flags are valid.
                unsafe { entry.set_flags(&flags) };
            }
        }

        let table = match self.entry_to_table(entry) {
            Ok(table) => table,
            Err(PageTableFrameError::HugePage) => return Err(MapToError::ParentHugePage),
            Err(PageTableFrameError::NotPresent) => unreachable!("entry was just set to present"),
        };

        if new_frame {
            table.clear();
        }

        Ok(table)
    }
}
