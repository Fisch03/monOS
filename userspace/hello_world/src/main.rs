#![no_std]
#![no_main]

use monos_std::{prelude::*, println, syscall::open_fb};

#[no_mangle]
fn main() {
    println!("{} {}", "good mononing!", 42);
}
