mod page;
pub use page::Page;

mod frame;
pub use frame::Frame;

mod page_table;
pub use page_table::PageTableFlags;
use page_table::{PageTable, PageTableIndex};

mod frame_allocator;
pub use frame_allocator::FrameAllocator;

mod mapper;
pub use mapper::{MapTo, Mapper};

mod pat;
pub use pat::PAT;

use crate::arch::registers::CR3;
use crate::mem::VirtualAddress;
use crate::utils::BitField;

pub trait PageSize: Copy {
    const SIZE: u64;
}

#[derive(Debug, Clone, Copy)]
pub struct PageSize4K;
impl PageSize for PageSize4K {
    const SIZE: u64 = 4096;
}

#[derive(Debug, Clone, Copy)]
pub struct PageSize2M;
impl PageSize for PageSize2M {
    const SIZE: u64 = 4096 * 512;
}

#[derive(Debug, Clone, Copy)]
pub struct PageSize1G;
impl PageSize for PageSize1G {
    const SIZE: u64 = 4096 * 512 * 512;
}

impl VirtualAddress {
    fn page_offset(&self) -> u64 {
        self.0.get_bits(0..12)
    }

    fn p1_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.0.get_bits(12..21) as u16)
    }

    fn p2_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.0.get_bits(21..30) as u16)
    }

    fn p3_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.0.get_bits(30..39) as u16)
    }

    fn p4_index(&self) -> PageTableIndex {
        PageTableIndex::new(self.0.get_bits(39..48) as u16)
    }
}

/// safety: the physical memory offset must be correct
pub unsafe fn active_level_4_table(physical_mem_offset: VirtualAddress) -> &'static mut PageTable {
    let (l4_table, _) = CR3::read();
    let phys = l4_table.start_address();
    let virt = physical_mem_offset + phys.as_u64();

    let ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *ptr
}
