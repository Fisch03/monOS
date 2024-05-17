mod page_table;
use page_table::{PageTable, PageTableIndex};

mod mapper;
use mapper::Mapper;

mod frame;
pub use frame::{Frame, FrameSize4K};

use crate::arch::registers::CR3;
use crate::mem::VirtualAddress;
use crate::utils::BitField;

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

/// safety: the physical memory offset must be correct and the page tables need to be set up correctly.
pub unsafe fn init(physical_mem_offset: VirtualAddress) -> Mapper<'static> {
    // safety: the caller guarantees that the physical memory offset is correct.
    let mapper = unsafe { Mapper::new(physical_mem_offset) };
    mapper
}

/// safety: the physical memory offset must be correct
pub unsafe fn active_level_4_table(physical_mem_offset: VirtualAddress) -> &'static mut PageTable {
    let (l4_table, _) = CR3::read();
    let phys = l4_table.start_address();
    let virt = physical_mem_offset + phys.as_u64();

    let ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *ptr
}
