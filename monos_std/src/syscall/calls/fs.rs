use super::*;
use crate::fs::*;
use crate::io::SeekMode;

use alloc::vec::Vec;
use core::mem::MaybeUninit;

pub fn open<'p, P: Into<Path<'p>>>(path: P, _flags: FileFlags) -> Option<FileHandle> {
    let path: Path = path.into();
    let path = path.as_str();

    let path_ptr = path.as_ptr() as u64;
    let path_len = path.len() as u64;

    let mut file_handle: Option<FileHandle> = None;

    let file_handle_ptr = &mut file_handle as *mut _;
    unsafe {
        syscall_3(
            Syscall::new(SyscallType::Open),
            path_ptr,
            path_len,
            file_handle_ptr as u64,
        );
    }

    file_handle
}

pub fn close(handle: &FileHandle) {
    unsafe {
        syscall_1(Syscall::new(SyscallType::Close), handle.as_u64());
    }
}

pub fn seek(handle: &FileHandle, offset: i64, mode: SeekMode) -> u64 {
    let mode: u8 = mode.into();
    let offset = offset as u64;
    let new_offset = unsafe {
        syscall_3(
            Syscall::new(SyscallType::Seek),
            handle.as_u64(),
            offset,
            mode as u64,
        )
    };
    new_offset
}

pub fn read(handle: &FileHandle, buf: &mut [u8]) -> usize {
    let buf_ptr = buf.as_mut_ptr() as u64;
    let buf_len = buf.len() as u64;

    let read = unsafe {
        syscall_3(
            Syscall::new(SyscallType::Read),
            handle.as_u64(),
            buf_ptr,
            buf_len,
        ) as usize
    };

    read
}

pub fn stat(_handle: &FileHandle) -> Option<FileInfo> {
    todo!();
}

pub fn list<'p, P: Into<Path<'p>>>(path: P) -> Vec<PathBuf> {
    let path: Path = path.into();
    let path = path.as_str();

    let path_ptr = path.as_ptr() as u64;
    let path_len = path.len() as u64;

    // TODO: do a stat first and then use a vec of appropriate size
    let mut paths: MaybeUninit<[ArrayPath; 5]> = MaybeUninit::uninit();
    let paths_ptr = &mut paths as *mut _;

    let amt = unsafe {
        syscall_3(
            Syscall::new(SyscallType::List),
            path_ptr,
            path_len,
            paths_ptr as u64,
        )
    };

    assert!(amt <= 5, "more than 5 paths returned, fixme!");

    // let paths_ptr = unsafe { VolatilePtr::new(NonNull::new(paths_ptr).unwrap()) };
    // let paths = paths_ptr.read();

    // safety: assuming the os does what its supposed to, the first `amt` slots are initialized, so
    // if we only take `amt` slots, we should be good
    let paths = unsafe { paths.assume_init() };
    paths
        .iter()
        .take(amt as usize)
        .map(|path| PathBuf::from_str(path.as_str()))
        .collect()
}
