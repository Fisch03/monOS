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

    Open,
    Seek,
    Read,
    Write,

    Print,
}

#[repr(C, packed)]
pub struct Syscall {
    pub ty: SyscallType,
    flags: u8,
    receiver_pid: u32,
    receiver_channel: u8,
    sender_channel: u8,
}

impl Syscall {
    pub const fn new(ty: SyscallType) -> Self {
        Self {
            ty,
            flags: 0,
            receiver_pid: 0,
            receiver_channel: 0,
            sender_channel: 0,
        }
    }

    #[inline(always)]
    pub fn get_handle(&self) -> ChannelHandle {
        ChannelHandle::new(
            self.receiver_pid,
            self.receiver_channel.into(),
            self.sender_channel.into(),
        )
    }

    pub fn with_handle(mut self, channel: ChannelHandle) -> Self {
        // TODO: raise channel id limit to 12 bits
        self.receiver_pid = channel.target_process;
        self.receiver_channel = channel
            .target_channel
            .try_into()
            .expect("channel id too large (fixme pls)");
        self.sender_channel = channel
            .own_channel
            .try_into()
            .expect("channel id too large (fixme pls)");
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
        let receiver_pid = self.receiver_pid;
        let receiver_channel = self.receiver_channel;
        let sender_channel = self.sender_channel;

        f.debug_struct("Syscall")
            .field("type", &self.ty)
            .field("flags", &self.flags)
            .field("receiver_pid", &receiver_pid)
            .field("receiver_channel", &receiver_channel)
            .field("sender_channel", &sender_channel)
            .finish()
    }
}

#[cfg(feature = "userspace")]
pub use calls::*;

#[cfg(feature = "userspace")]
mod calls {
    use super::*;

    use core::arch::asm;

    mod fs;
    mod ipc;

    pub use fs::*;
    pub use ipc::*;

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
                out("rcx") _, out("r11") _, out("r9") _
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
                out("rcx") _, out("r11") _, out("r9") _

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
                out("rcx") _, out("r11") _, out("r9") _

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
                out("rcx") _, out("r11") _, out("r9") _

            );
        }
        ret
    }
}
