use crate::mem::{alloc_frame, map_to, Page, PageTableFlags};

use linked_list_allocator::LockedHeap;
// use buddy_system_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use crate::HEAP_START;
// const HEAP_SIZE: u64 = 4096 * 1024; // 4 MiB
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
        let frame = alloc_frame("heap").expect("failed to allocate frame for heap");

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { map_to(&start_page, &frame, flags).unwrap() };

        start_page = start_page.next();
    }

    unsafe { core::ptr::write_bytes(HEAP_START.as_mut_ptr::<u8>(), 0, HEAP_SIZE as usize) };

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START.as_mut_ptr(), HEAP_SIZE as usize);
    }
}

mod test {
    use monos_test::kernel_test;

    #[kernel_test]
    fn test_heap_alloc_all(boot_info: &bootloader_api::BootInfo) -> bool {
        use alloc::vec::Vec;

        unsafe { crate::mem::init(boot_info) };

        let allocations = super::HEAP_SIZE / core::mem::size_of::<usize>() as u64;

        let mut vec = Vec::new();
        for i in 0..allocations {
            if i % 100 == 0 {
                crate::print!("allocating {}/{}\x1b[0K...\r", i, allocations);
            }
            vec.push(42);
        }

        for n in &vec {
            if *n != 42 {
                crate::println!("failed to allocate all heap memory");
                return false;
            }
        }

        crate::print!("\n");

        crate::println!("deallocating all heap memory...");
        vec.clear();
        vec.shrink_to_fit();

        true
    }
}
