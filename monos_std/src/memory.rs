use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub(crate) unsafe fn init(heap_start: usize, heap_size: usize) {
    // sometimes the heap is already locked (probably due to some non-zeroed memory?). we just
    // force-unlock it here since it wasn't used by anyone before.
    unsafe { ALLOCATOR.force_unlock() };
    unsafe { ALLOCATOR.lock().init(heap_start as *mut u8, heap_size) };
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
