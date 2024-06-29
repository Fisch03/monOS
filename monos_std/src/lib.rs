#![no_std]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]

extern crate alloc;

#[cfg(not(feature = "lib_only"))]
use core::arch::asm;

#[cfg(not(feature = "lib_only"))]
mod memory;

pub mod messaging;

pub mod dev;
pub mod syscall;

pub use prelude::*;

pub mod prelude {
    pub use crate::{dbg, messaging::MessageData, print, println, syscall};
    pub use alloc::{
        boxed::Box,
        //format, // format!() causes a page fault for some reason
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    pub use core::prelude::rust_2021::*;
}

extern "C" {
    fn main();
}

#[cfg(not(feature = "lib_only"))]
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
    #[cfg(not(feature = "lib_only"))]
    unsafe {
        memory::init(heap_start, heap_size)
    };

    unsafe { main() };

    // TODO: exit syscall
}

#[cfg(not(feature = "lib_only"))]
#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(feature = "lib_only"))]
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use arrayvec::ArrayString;
    use core::fmt::Write;

    let mut message = ArrayString::<128>::new();
    write!(message, "oh noes! the program {}", info).unwrap();
    println!("{}", message);

    // TODO: exit syscall

    loop {}
}
