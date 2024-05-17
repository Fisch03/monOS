mod page_table;
use page_table::{PageTable, PageTableIndex};

use crate::arch::registers::CR3;
use crate::mem::{PhysicalAddress, VirtualAddress};
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

pub unsafe fn active_level_4_table(physical_mem_offset: VirtualAddress) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (l4_table, _) = Cr3::read();
    let phys = l4_table.start_address();
    let virt = physical_mem_offset + phys.as_u64();

    let ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *ptr
}

pub unsafe fn translate_addr(
    addr: VirtualAddress,
    physical_mem_offset: VirtualAddress,
) -> Option<PhysicalAddress> {
    translate_addr_inner(addr, physical_mem_offset)
}

pub fn translate_addr_inner(
    addr: VirtualAddress,
    physical_mem_offset: VirtualAddress,
) -> Option<PhysicalAddress> {
    let (mut frame, _) = CR3::read();

    let table_indices = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    for table_index in table_indices {
        let virt = physical_mem_offset + frame.start_address().as_u64();
        let ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*ptr };

        let entry = &table[table_index];
        frame = entry.frame()?;
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
