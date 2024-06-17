use core::arch::asm;

mod io;
pub use io::*;

mod gfx;
pub use gfx::*;

#[derive(Debug)]
#[repr(u64)]
pub enum Syscall {
    Print = 0,
    OpenFramebuffer = 1,
    SubmitFrame = 2,
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn syscall_1(syscall: Syscall, arg1: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall as u64,
            in("rdi") arg1,
            lateout("rax") ret,
        );
    }
    ret
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn syscall_2(syscall: Syscall, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall as u64,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rax") ret,
        );
    }
    ret
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn syscall_3(syscall: Syscall, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall as u64,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") ret,
        );
    }
    ret
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn syscall_4(syscall: Syscall, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall as u64,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            lateout("rax") ret,
        );
    }
    ret
}
