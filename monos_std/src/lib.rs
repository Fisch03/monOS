#![no_std]
#![feature(prelude_import)]

extern crate alloc;

use core::arch::asm;

mod memory;
pub mod syscall;
pub use monos_gfx as gfx;

#[allow(unused_imports)]
#[prelude_import]
pub use prelude::*;

pub mod prelude {
    pub use crate::{print, println};
    pub use alloc::{
        boxed::Box,
        format,
        string::{String, ToString},
        vec::Vec,
    };
    pub use core::prelude::rust_2021::*;
}

extern "C" {
    fn main();
}

#[no_mangle]
pub extern "sysv64" fn _start() -> ! {
    let heap_start: usize;
    let heap_size: usize;
    unsafe {
        asm!("",
             lateout("r10") heap_start,
             lateout("r11") heap_size,
             options(pure, nomem, nostack)
        )
    };

    unsafe { memory::init(heap_start, heap_size) };

    unsafe { main() };

    // TODO: exit syscall
    loop {}
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO

    syscall::print("userspace program panicked!");
    //println!("oh noes! the program {}", info);
    loop {}
}
