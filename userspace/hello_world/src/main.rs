#![no_std]
#![no_main]

use monos_std::syscall;

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    syscall::print("Hello, world!\n");

    loop {}
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
