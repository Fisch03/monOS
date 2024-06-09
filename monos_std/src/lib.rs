#![no_std]

extern crate alloc;

mod memory;
pub mod syscall;
pub use monos_gfx as gfx;

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    extern "C" {
        fn main();
    }

    // syscall::print("Hello, world!\n");
    unsafe { main() };

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
