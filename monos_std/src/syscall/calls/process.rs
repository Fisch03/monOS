use super::*;
use crate::fs::*;
use crate::ProcessId;

pub fn spawn<'p, P: Into<Path<'p>>>(path: P) -> Option<ProcessId> {
    let path: Path = path.into();
    let path = path.as_str();

    let path_ptr = path.as_ptr() as u64;
    let path_len = path.len() as u64;

    let ret = unsafe { syscall_4(Syscall::new(SyscallType::Spawn), path_ptr, path_len, 0, 0) };

    if ret == 0 {
        None
    } else {
        Some(ProcessId(ret as u32))
    }
}

pub fn spawn_with_args<'p, P: Into<Path<'p>>>(path: P, args: &str) -> Option<ProcessId> {
    let path: Path = path.into();
    let path = path.as_str();

    let path_ptr = path.as_ptr() as u64;
    let path_len = path.len() as u64;

    let args_ptr = args.as_ptr() as u64;
    let args_len = args.len() as u64;

    let ret = unsafe {
        syscall_4(
            Syscall::new(SyscallType::Spawn),
            path_ptr,
            path_len,
            args_ptr,
            args_len,
        )
    };

    if ret == 0 {
        None
    } else {
        Some(ProcessId(ret as u32))
    }
}

pub fn yield_() {
    unsafe {
        syscall_0(Syscall::new(SyscallType::Yield));
    }
}
