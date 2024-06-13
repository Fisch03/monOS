#![no_std]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]

extern crate alloc;

use core::arch::asm;

mod memory;
pub mod syscall;
pub use monos_gfx as gfx;

pub use prelude::*;

pub mod prelude {
    pub use crate::{dbg, print, println, syscall};
    pub use alloc::{
        boxed::Box,
        //format, // format!() causes a page fault for some reason
        string::{String, ToString},
        vec::Vec,
    };
    pub use core::prelude::rust_2021::*;
}

extern "C" {
    fn main();
}

#[no_mangle]
#[naked]
pub unsafe extern "sysv64" fn _start() -> ! {
    asm!(
        "and rsp, -16",

        "mov rdi, r10",
        "mov rsi, r11",
        "call {start_inner}",
        "2:",
        "jmp 2b",

        start_inner = sym start_inner,
        options(noreturn)
    )
}

#[inline(never)]
extern "C" fn start_inner(heap_start: usize, heap_size: usize) {
    unsafe { memory::init(heap_start, heap_size) };

    unsafe { main() };

    // TODO: exit syscall
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("oh noes! the program {}", info);

    // TODO: exit syscall

    loop {}
}
