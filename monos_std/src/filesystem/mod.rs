mod path;
pub use path::*;

use crate::syscall;

#[derive(Debug, Clone, Copy)]
pub struct FileHandle(u64);

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: usize,
}

impl FileHandle {
    pub const fn new(fd: u64) -> Self {
        Self(fd)
    }

    #[cfg(not(feature = "lib_only"))]
    pub fn open<'p, P: Into<Path<'p>>>(path: P) -> Option<Self> {
        syscall::open(path.into(), FileFlags)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[cfg(not(feature = "lib_only"))]
    pub fn stat(&self) -> Option<FileInfo> {
        syscall::stat(&self)
    }
}

#[cfg(not(feature = "lib_only"))]
impl Read for FileHandle {
    fn read(&self, buf: &mut [u8]) -> usize {
        syscall::read(&self, buf)
    }
}

#[cfg(not(feature = "lib_only"))]
impl Write for FileHandle {
    fn write(&mut self, buf: &[u8]) -> usize {
        todo!()
    }
}

#[cfg(not(feature = "lib_only"))]
impl Seek for FileHandle {
    fn seek(&self, pos: usize) {
        todo!()
    }
}

pub struct FileFlags;

pub trait Read {
    fn read(&self, buf: &mut [u8]) -> usize;

    fn read_all(&self, buf: &mut [u8]) -> usize {
        let mut total_read = 0;
        while total_read < buf.len() {
            let read = self.read(&mut buf[total_read..]);
            if read == 0 {
                break;
            }
            total_read += read;
        }
        total_read
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> usize;

    fn write_all(&mut self, buf: &[u8]) -> usize {
        let mut total_written = 0;
        while total_written < buf.len() {
            let written = self.write(&buf[total_written..]);
            if written == 0 {
                break;
            }
            total_written += written;
        }
        total_written
    }
}

pub trait Seek {
    fn seek(&self, pos: usize);
}
