#![no_std]
#![feature(alloc_error_handler)]
#![feature(prelude_import)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]

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
    // let rsp: usize;
    // unsafe { asm!("mov {}, rsp", lateout(reg) rsp, options(nostack)) };
    unsafe { memory::init(heap_start, heap_size) };

    {
        let test = Box::new(42);
        assert_eq!(*test, 42);
    }

    println!("heap_size: {:#x}, heap_start: {:#x}", heap_size, heap_start);

    // unsafe { main() };

    // TODO: exit syscall
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO

    let mut panic_buf: arrayvec::ArrayString<512> = arrayvec::ArrayString::new();
    use core::fmt::Write;
    write!(panic_buf, "{}", info).unwrap();
    syscall::print(panic_buf.as_str());
    //println!("oh noes! the program {}", info);
    loop {}
}
