mod path;
#[cfg(feature = "userspace")]
use crate::io::{Read, Seek, Write};
pub use path::*;

#[cfg(feature = "userspace")]
use crate::syscall;

#[derive(Debug, Clone, Copy)]
pub struct File(u64);

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: usize,
}

impl File {
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
impl Read for File {
    fn read(&self, buf: &mut [u8]) -> usize {
        syscall::read(&self, buf)
    }
}

#[cfg(feature = "userspace")]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> usize {
        todo!()
    }
}

#[cfg(feature = "userspace")]
impl Seek for File {
    fn seek(&self, pos: usize) {
        todo!()
    }
}

pub struct FileFlags;
