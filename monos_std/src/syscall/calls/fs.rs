use super::*;
use crate::filesystem::*;

use core::ptr::NonNull;
use volatile::VolatilePtr;

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

    let file_handle_ptr = unsafe { VolatilePtr::new(NonNull::new(file_handle_ptr).unwrap()) };
    file_handle_ptr.read()
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

    crate::println!("read: {:?}", read);

    read
}

pub fn stat(handle: &FileHandle) -> Option<FileInfo> {
    todo!();
}
