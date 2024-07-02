use super::*;
use crate::filesystem::*;

pub fn open<'p, P: Into<Path<'p>>>(path: P, flags: FileFlags) -> Option<FileHandle> {
    todo!();
}

pub fn read(handle: FileHandle, buf: &mut [u8]) -> usize {
    todo!();
}
