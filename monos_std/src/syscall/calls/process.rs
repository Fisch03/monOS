use super::*;
use crate::fs::*;
use crate::ProcessId;

pub fn spawn<'p, P: Into<Path<'p>>>(path: P) -> Option<ProcessId> {
    let path: Path = path.into();
    let path = path.as_str();

    let path_ptr = path.as_ptr() as u64;
    let path_len = path.len() as u64;

    let ret = unsafe { syscall_2(Syscall::new(SyscallType::Spawn), path_ptr, path_len) };

    if ret == 0 {
        None
    } else {
        Some(ProcessId(ret as u32))
    }
}
