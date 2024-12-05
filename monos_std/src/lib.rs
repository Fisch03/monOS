#![no_std]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(maybe_uninit_uninit_array)]

extern crate alloc;

#[cfg(feature = "userspace")]
use core::arch::naked_asm;

#[cfg(feature = "userspace")]
mod memory;

pub mod io;

pub mod fs;
pub mod messaging;

pub mod dev;

#[cfg(any(feature = "userspace", feature = "syscall"))]
pub mod syscall;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct ProcessId(pub u32);

impl ProcessId {
    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub use prelude::*;

pub mod prelude {
    pub fn args() -> &'static Vec<String> {
        #[allow(static_mut_refs)]
        unsafe {
            crate::ARGS.as_ref().unwrap()
        }
    }

    pub use crate::fs::{self, FileHandle as File, Path, PathBuf};
    pub use crate::io::{Read, Seek, Write};
    pub use crate::messaging::MessageData;
    pub use crate::ProcessId;

    #[cfg(feature = "syscall")]
    pub use crate::syscall;
    #[cfg(feature = "syscall")]
    pub use crate::syscall::SysInfo;

    #[cfg(feature = "userspace")]
    pub use crate::{dbg, print, println};

    pub use alloc::{
        boxed::Box,
        collections,
        format, // format!() causes a page fault for some reason
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    pub use core::prelude::rust_2021::*;
}

static mut ARGS: Option<Vec<String>> = None;

extern "C" {
    fn main();
}

#[cfg(feature = "userspace")]
#[no_mangle]
#[naked]
pub unsafe extern "sysv64" fn _start() -> ! {
    naked_asm!(
        "and rsp, -16",
        //"sub rsp, 8", // align stack to 16 bytes

        "mov rdi, r10",
        "mov rsi, r11",
        "call {start_inner}",
        "2:",
        "jmp 2b",

        start_inner = sym start_inner,
    )
}

#[inline(never)]
#[allow(dead_code)]
extern "C" fn start_inner(_heap_start: usize, _heap_size: usize) {
    let args_ptr: *const u64;

    unsafe {
        core::arch::asm!("",
            lateout("rdx") args_ptr,
            options(pure, nomem, nostack),
        )
    };

    #[cfg(feature = "userspace")]
    unsafe {
        memory::init(_heap_start, _heap_size)
    };

    let args = if args_ptr.is_null() {
        Vec::new()
    } else {
        let len = unsafe { *(args_ptr as *const u64) };

        let args = unsafe {
            core::str::from_utf8(core::slice::from_raw_parts(
                args_ptr.add(1) as *const u8,
                len as usize,
            ))
        }
        .expect("args are an invalid utf8 string");

        args.split(' ').map(|s| s.into()).collect()
    };

    unsafe {
        ARGS = Some(args);
    }

    unsafe { main() };

    // TODO: exit syscall
}

#[cfg(feature = "userspace")]
#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(feature = "userspace")]
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use arrayvec::ArrayString;
    use core::fmt::Write;

    let mut message = ArrayString::<256>::new();
    write!(message, "oh noes! the program {}", info).unwrap();
    println!("{}", message);

    // TODO: exit syscall

    loop {
        syscall::yield_();
    }
}
