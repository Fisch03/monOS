#![no_std]
#![no_main]

use monos_std::syscall;

const HELLO: &str = "Hello, world!\n";

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    loop {
        syscall::print(HELLO);
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
