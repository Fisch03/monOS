use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::messaging::ChannelHandle;
use crate::ProcessId;

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum SyscallType {
    Spawn = 0,
    Yield,

    Serve,
    Connect,
    WaitConnect,
    Send,
    Receive,
    ReceiveAny,

    RequestChunk,

    Open,
    Close,
    Seek,
    Read,
    Write,

    List,

    Print,
    SysInfo,
}

#[repr(C, packed)]
pub struct Syscall {
    pub ty: SyscallType,
    flags: SyscallFlags,
    receiver_pid: ProcessId,
    receiver_channel: u8,
    sender_channel: u8,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SyscallFlags(u8);

impl SyscallFlags {
    const IS_CHUNK: u8 = 1 << 0;
    const IS_MMAPPED: u8 = 1 << 1;

    const fn new() -> Self {
        Self(0)
    }

    pub fn is_chunk(&self) -> bool {
        self.0 & Self::IS_CHUNK != 0
    }
    pub fn is_mmapped(&self) -> bool {
        self.0 & Self::IS_MMAPPED != 0
    }
}

#[cfg(feature = "userspace")]
impl SyscallFlags {
    fn set_is_chunk(&mut self) {
        self.0 |= Self::IS_CHUNK;
    }

    fn set_is_mmapped(&mut self) {
        self.0 |= Self::IS_MMAPPED;
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl core::fmt::Debug for SyscallFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SyscallFlags")
            .field("is_chunk", &self.is_chunk())
            .field("dont_unmap", &self.is_mmapped())
            .finish()
    }
}

impl Syscall {
    pub const fn new(ty: SyscallType) -> Self {
        Self {
            ty,
            flags: SyscallFlags::new(),
            receiver_pid: ProcessId(0),
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

    #[inline(always)]
    pub fn flags(&self) -> SyscallFlags {
        self.flags
    }
    #[inline(always)]
    pub fn with_flags(mut self, flags: SyscallFlags) -> Self {
        self.flags = flags;
        self
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

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u64)]
pub enum SysInfo {
    SystemTime,

    FreeMemory,
    UsedMemory,
    TotalMemory,

    ProcessId,
    NumProcesses,
    // OsVersion,
}

#[cfg(feature = "userspace")]
pub use calls::*;

#[cfg(feature = "userspace")]
mod calls {
    use super::*;

    use core::arch::asm;

    mod fs;
    mod ipc;
    mod os;
    mod process;

    pub use fs::*;
    pub use ipc::*;
    pub use os::*;
    pub use process::*;

    #[inline(always)]
    #[allow(dead_code)]
    unsafe fn syscall_0(syscall: Syscall) -> u64 {
        let syscall: u64 = syscall.into();
        let ret: u64;
        unsafe {
            asm!(
                "syscall",
                in("rax") syscall,
                lateout("rax") ret,
                out("rcx") _, out("r11") _, out("r9") _,
            );
        }
        ret
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
                out("rcx") _, out("r11") _, out("r9") _,

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
