mod path;
pub use path::*;

#[cfg(feature = "userspace")]
use crate::io::{Read, Seek, SeekMode, Write};

#[cfg(feature = "userspace")]
use crate::syscall;

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct FileHandle(u64);

#[derive(Debug, Clone)]
pub enum FileInfo {
    File { size: usize },
    Directory { num_files: usize },
}

impl FileHandle {
    pub const fn new(fd: u64) -> Self {
        Self(fd)
    }

    #[cfg(feature = "userspace")]
    pub fn open<'p, P: Into<Path<'p>>>(path: P) -> Option<Self> {
        syscall::open(path.into(), FileFlags)
    }

    /// close the file handle.
    /// doesnt actually need to be called since the file will be closed when the handle is dropped.
    /// but can be used to close the file handle early or be a bit more explicit
    #[cfg(feature = "userspace")]
    pub fn close(self) {
        core::mem::drop(self);
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "userspace")]
    pub fn stat(&self) -> Option<FileInfo> {
        syscall::stat(&self)
    }
}

#[cfg(feature = "userspace")]
impl core::ops::Drop for FileHandle {
    fn drop(&mut self) {
        syscall::close(self);
    }
}

#[cfg(not(feature = "userspace"))]
impl Clone for FileHandle {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[cfg(not(feature = "userspace"))]
impl Copy for FileHandle {}

#[cfg(feature = "userspace")]
impl Read for FileHandle {
    fn read(&self, buf: &mut [u8]) -> usize {
        syscall::read(&self, buf)
    }
}

#[cfg(feature = "userspace")]
impl Write for FileHandle {
    fn write(&mut self, _buf: &[u8]) -> usize {
        todo!()
    }
}

#[cfg(feature = "userspace")]
impl Seek for FileHandle {
    fn set_pos(&self, pos: usize) {
        syscall::seek(self, pos as i64, SeekMode::Start);
    }

    fn get_pos(&self) -> usize {
        syscall::seek(self, 0, SeekMode::Current) as usize
    }

    fn seek(&self, offset: i64, mode: SeekMode) -> usize {
        syscall::seek(self, offset, mode) as usize
    }

    fn max_pos(&self) -> usize {
        usize::MAX
    }
}

pub struct FileFlags;
