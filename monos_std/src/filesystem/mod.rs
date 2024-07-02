mod path;
pub use path::*;

pub struct FileHandle(u64);

impl FileHandle {
    pub const fn new(fd: u64) -> Self {
        Self(fd)
    }
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

pub struct FileFlags;
