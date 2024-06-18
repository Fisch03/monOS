use core::arch::asm;

mod io;
pub use io::*;

mod gfx;
pub use gfx::*;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::messaging::ChannelHandle;

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum SyscallType {
    Serve = 0,
    Connect,
    WaitConnect,
    Send,
    SendSync,
    Receive,
    ReceiveAny,

    Print,

    OpenFramebuffer,
    SubmitFrame,
}

#[repr(packed)]
pub struct Syscall {
    pub ty: SyscallType,
    _reserved1: u8,
    channel_pid: u32,
    channel_number: u16,
}

impl Syscall {
    pub const fn new(ty: SyscallType) -> Self {
        Self {
            ty,
            _reserved1: 0,
            channel_pid: 0,
            channel_number: 0,
        }
    }

    #[inline(always)]
    pub fn get_handle(&self) -> ChannelHandle {
        ChannelHandle::new(self.channel_pid, self.channel_number)
    }

    pub fn with_handle(mut self, channel: ChannelHandle) -> Self {
        self.channel_pid = channel.thread();
        self.channel_number = channel.channel();
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
            .field("type", &self.ty)
            .field("handle", &self.get_handle())
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
