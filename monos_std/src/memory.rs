use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub(crate) unsafe fn init(heap_start: usize, heap_size: usize) {
    let heap_start = heap_start as *mut u8;
    unsafe {*heap_start = 0x12};
    //unsafe { ALLOCATOR.lock().init(heap_start as *mut u8, heap_size) }
}
