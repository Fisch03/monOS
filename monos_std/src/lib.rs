#![no_std]
#![no_main]

extern crate alloc;

mod memory;
pub mod syscall;
pub use monos_gfx as gfx;

extern "C" {
    fn main(argc: isize, argv: *const *const u8) -> isize;
}

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    syscall::print("Hello, world!\n");
    unsafe { main(0, core::ptr::null()) };

    // TODO: exit syscall
    loop {}
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // TODO
    syscall::print("panic!\n");
    loop {}
}
