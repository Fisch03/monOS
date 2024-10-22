#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use core::arch::asm;

#[no_mangle]
fn main() {
    println!("terminal!");
    loop {
        syscall::yield_();
    }
}
