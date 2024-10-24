mod path;
pub use path::*;

#[cfg(feature = "userspace")]
use crate::io::{Read, Seek, Write};

#[cfg(feature = "userspace")]
use crate::syscall;

#[derive(Debug, Clone, Copy)]
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

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "userspace")]
    pub fn stat(&self) -> Option<FileInfo> {
        syscall::stat(&self)
    }
}

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
    fn seek(&self, _pos: usize) {
        todo!()
    }
}

pub struct FileFlags;
