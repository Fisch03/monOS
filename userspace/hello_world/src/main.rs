#![no_std]
#![no_main]

use monos_std::syscall;

#[no_mangle]
fn main() {
    syscall::print("good mononing!\n");
}
