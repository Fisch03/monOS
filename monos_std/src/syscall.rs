use core::arch::asm;

#[derive(Debug)]
#[repr(u64)]
pub enum Syscall {
    Print = 0,
}

impl core::convert::TryFrom<u64> for Syscall {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Syscall::Print),
            _ => Err(()),
        }
    }
}

pub fn print(s: &str) {
    let ptr = s.as_ptr() as u64;
    let len = s.len() as u64;
    syscall_2(Syscall::Print, ptr, len);
}

#[inline]
#[allow(dead_code)]
fn syscall_1(syscall: Syscall, arg1: u64) -> u64 {
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

#[inline]
#[allow(dead_code)]
fn syscall_2(syscall: Syscall, arg1: u64, arg2: u64) -> u64 {
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

#[inline]
#[allow(dead_code)]
fn syscall_3(syscall: Syscall, arg1: u64, arg2: u64, arg3: u64) -> u64 {
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

#[inline]
#[allow(dead_code)]
fn syscall_4(syscall: Syscall, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
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
