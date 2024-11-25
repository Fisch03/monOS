use core::ffi::c_void;

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let layout = core::alloc::Layout::from_size_align(size, 1).unwrap();
    alloc::alloc::alloc(layout) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }

    let layout = core::alloc::Layout::from_size_align(1, 1).unwrap();
    alloc::alloc::dealloc(ptr as *mut u8, layout);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    let layout = core::alloc::Layout::from_size_align(nmemb * size, 1).unwrap();
    alloc::alloc::alloc_zeroed(layout) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    let layout = core::alloc::Layout::from_size_align(size, 1).unwrap();
    alloc::alloc::realloc(ptr as *mut u8, layout, size) as *mut c_void
}
