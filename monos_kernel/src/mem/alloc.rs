use crate::mem::{alloc_frame, map_to, Page, PageTableFlags, VirtualAddress};

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_START: u64 = 0x_4444_4444_0000;
const HEAP_SIZE: u64 = 1024 * 1024; // 1 MiB

pub fn init_heap() {
    let heap_start = VirtualAddress::new(HEAP_START);
    let heap_end = heap_start + HEAP_SIZE;
    let mut start_page = Page::around(heap_start);
    let end_page = Page::around(heap_end);

    while start_page != end_page {
        let frame = alloc_frame().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { map_to(&start_page, &frame, flags).unwrap() };

        start_page = start_page.next();
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start.as_mut_ptr(), HEAP_SIZE as usize);
    }
}
