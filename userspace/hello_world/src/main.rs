#![no_std]
#![no_main]

use monos_std::syscall;

#[no_mangle]
extern "C" fn main() {
    loop {
        syscall::print("good mononing!\n");
    }
}
