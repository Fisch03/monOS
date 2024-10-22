use crate::mem::{alloc_frame, map_to, Page, PageTableFlags};

use linked_list_allocator::LockedHeap;
// use buddy_system_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// const HEAP_SIZE: u64 = 4096 * 1024; // 4 MiB
use crate::HEAP_START;
const HEAP_SIZE: u64 = 4096 * 4096; // 16 MiB

pub fn init() {
    let heap_end = HEAP_START + HEAP_SIZE;
    let mut start_page = Page::around(HEAP_START);
    let end_page = Page::around(heap_end);

    crate::println!(
        "allocating heap from {:#x} to {:#x}",
        HEAP_START.as_u64(),
        heap_end.as_u64()
    );

    while start_page != end_page {
        let frame = alloc_frame().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { map_to(&start_page, &frame, flags).unwrap() };

        start_page = start_page.next();
    }

    unsafe {
        ALLOCATOR
            .lock()
            // .init(heap_start.as_u64() as usize, HEAP_SIZE as usize);
            .init(HEAP_START.as_mut_ptr(), HEAP_SIZE as usize);
    }
}
