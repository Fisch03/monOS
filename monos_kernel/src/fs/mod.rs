mod path;
pub use path::*;

pub mod fat16;
use alloc::string::String;
use fat16::Fat16Fs;

mod ramdisk;
use ramdisk::RamDisk;

use crate::mem::VirtualAddress;
use bootloader_api::BootInfo;

pub fn init(boot_info: &BootInfo) {
    let ramdisk_start = VirtualAddress::new(
        boot_info
            .ramdisk_addr
            .into_option()
            .expect("no ramdisk found"),
    );
    let ramdisk_size = boot_info.ramdisk_len;

    let ram_disk = unsafe { RamDisk::new(ramdisk_start, ramdisk_size as usize) };
    let mut fs = Fat16Fs::new(ram_disk).expect("failed to initialize FAT16 filesystem");

    for entry in fs.iter_root_dir() {
        crate::dbg!(entry.name());
    }

    let welcome = fs
        .iter_root_dir()
        .get_entry("home/welcome.md")
        .expect("no home directory");
    crate::dbg!(welcome.name());

    let welcome = welcome.as_file().expect("not a file");
    let mut buf = [0; 1024];
    let read = welcome.read_all(&mut buf);
    crate::dbg!(core::str::from_utf8(&buf[..read]).unwrap());
}

#[derive(Debug)]
pub enum GetFileError {
    InvalidPath,
    NotFound,
    NotADirectory,
}

pub trait File: Read + Write + Seek {
    fn name(&self) -> &str;
    fn size(&self) -> usize;
}

pub trait DirEntry: Sized {
    type File: File;
    type DirIter: Iterator<Item = Self> + DirIter<Item = Self>;

    fn name(&self) -> &str;
    fn is_dir(&self) -> bool;

    fn as_file(&self) -> Option<Self::File>;

    fn iter(&self) -> Option<Self::DirIter>;

    fn get_entry<'p, P: Into<Path<'p>>>(&self, path: P) -> Result<Self, GetFileError> {
        let mut iter = self.iter().ok_or(GetFileError::NotADirectory)?;
        iter.get_entry(path)
    }
}

pub trait DirIter: Iterator + Sized
where
    Self::Item: DirEntry,
{
    fn get_entry<'p, P: Into<Path<'p>>>(&mut self, path: P) -> Result<Self::Item, GetFileError> {
        let path = path.into();
        if let Some((current_dir, children)) = path.enter() {
            for entry in self {
                if entry.name() == current_dir.as_str() {
                    return entry.get_entry(children);
                }
            }
            Err(GetFileError::NotFound)
        } else {
            for entry in self {
                if entry.name() == path.as_str() {
                    return Ok(entry);
                }
            }
            Err(GetFileError::NotFound)
        }
    }
}

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
