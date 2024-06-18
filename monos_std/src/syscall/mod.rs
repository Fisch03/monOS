use core::arch::asm;

mod io;
pub use io::*;

mod gfx;
pub use gfx::*;

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum SyscallType {
    Serve,
    Send,
    Receive,
    Print,
    OpenFramebuffer,
    SubmitFrame,
}

#[repr(packed)]
pub struct Syscall {
    pub ty: SyscallType,
    port_len: u8,
    port_addr: u32,
    _reserved: u16,
}

impl Syscall {
    pub const fn new(ty: SyscallType) -> Self {
        Self {
            ty,
            port_len: 0,
            port_addr: 0,
            _reserved: 0,
        }
    }

    #[inline(always)]
    pub fn get_port(&self) -> Option<&str> {
        if self.port_len == 0 || self.port_addr == 0 {
            return None;
        }

        // safety:: we checked this when creating the syscall
        Some(unsafe {
            core::str::from_utf8(core::slice::from_raw_parts(
                self.port_addr as *const u8,
                self.port_len as usize,
            ))
            .ok()?
        })
    }

    pub fn with_port(mut self, port: &str) -> Self {
        assert!(port.len() < u8::MAX as usize);
        assert!((port.as_ptr() as u64) <= (u32::MAX as u64));

        self.port_len = port.len() as u8;
        self.port_addr = port.as_ptr() as u32;
        self
    }
}

impl From<Syscall> for u64 {
    fn from(syscall: Syscall) -> u64 {
        // safety: any value is valid for a u64
        unsafe { core::mem::transmute(syscall) }
    }
}

impl TryFrom<u64> for Syscall {
    type Error = ();
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        SyscallType::try_from_primitive(value as u8).map_err(|_| ())?;

        // safety: we just checked that the value is valid
        let syscall: Syscall = unsafe { core::mem::transmute(value) };

        Ok(syscall)
    }
}

impl core::fmt::Debug for Syscall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Syscall")
            .field("ty", &self.ty)
            .field("port", &self.get_port())
            .finish()
    }
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn syscall_1(syscall: Syscall, arg1: u64) -> u64 {
    let syscall: u64 = syscall.into();
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall,
            in("rdi") arg1,
            lateout("rax") ret,
        );
    }
    ret
}

#[inline(always)]
#[allow(dead_code)]
unsafe fn syscall_2(syscall: Syscall, arg1: u64, arg2: u64) -> u64 {
    let syscall: u64 = syscall.into();
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall,
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
    let syscall: u64 = syscall.into();
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall,
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
    let syscall: u64 = syscall.into();
    let ret: u64;
    unsafe {
        asm!(
            "syscall",
            in("rax") syscall,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            lateout("rax") ret,
        );
    }
    ret
}
