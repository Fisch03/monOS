use crate::mem::{
    active_level_4_table, alloc_frame, alloc_vmem, map_to, physical_mem_offset, Page, PageTable,
    PageTableFlags, VirtualAddress,
};

struct Process {
    id: usize,
    entry_point: VirtualAddress,
}

pub fn spawn(entry_point: unsafe fn()) {
    let page_table_frame = alloc_frame().expect("failed to alloc frame for process page table");
    let page_table_page = Page::around(unsafe { alloc_vmem(4096).align_up(4096) });
    unsafe {
        map_to(
            &page_table_page,
            &page_table_frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("failed to map page table page");
    };

    let kernel_page_table = active_level_4_table();

    //safety: this page table is invalid right now, but we overwrite it before it's used
    let process_page_table = page_table_page.start_address().as_mut_ptr::<PageTable>();
    let process_page_table = unsafe { &mut *process_page_table };

    process_page_table
        .iter_mut()
        .zip(kernel_page_table.iter())
        .for_each(|(process_entry, kernel_entry)| {
            *process_entry = kernel_entry.clone();
        });
}
