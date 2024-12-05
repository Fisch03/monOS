mod path;
pub use path::*;

#[cfg(feature = "userspace")]
use crate::{
    alloc::{
        string::{FromUtf8Error, String},
        vec::Vec,
    },
    io::{Read, Seek, SeekMode, Write},
};

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

    #[cfg(feature = "userspace")]
    pub fn size(&self) -> usize {
        let pos = self.get_pos();

        let size = self.seek(0, SeekMode::End) as usize;
        let _ = self.seek(pos as i64, SeekMode::Start);

        size
    }

    #[cfg(feature = "userspace")]
    pub fn read_to_vec(&self) -> Vec<u8> {
        let size = self.size();
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve_exact(size);
        unsafe { buf.set_len(size) }
        let read = self.read(buf.as_mut_slice());
        buf.truncate(read);
        buf
    }

    #[cfg(feature = "userspace")]
    pub fn read_to_string(&self) -> Result<String, FromUtf8Error> {
        let buf = self.read_to_vec();
        String::from_utf8(buf)
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
