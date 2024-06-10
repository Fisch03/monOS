use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub(crate) unsafe fn init(heap_start: usize, heap_size: usize) {
    unsafe { ALLOCATOR.lock().init(heap_start as *mut u8, heap_size) }
}
