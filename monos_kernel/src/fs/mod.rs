pub use monos_std::{
    filesystem::{Path, PathBuf},
    io::{Read, Seek, Write},
};

pub mod fat16;
use fat16::Fat16Fs;

mod ramdisk;
use ramdisk::RamDisk;

use crate::mem::VirtualAddress;
use bootloader_api::BootInfo;

use spin::Once;

// TODO: the fs impl currently has no safety at all in regards to opening the same file multiple times. that should definitely be added at some point but for now we ball

static FILESYSTEM: Once<Fat16Fs> = Once::new();

pub fn init(boot_info: &BootInfo) {
    FILESYSTEM.call_once(|| {
        let ramdisk_start = VirtualAddress::new(
            boot_info
                .ramdisk_addr
                .into_option()
                .expect("no ramdisk found"),
        );
        let ramdisk_size = boot_info.ramdisk_len;

        let ram_disk = unsafe { RamDisk::new(ramdisk_start, ramdisk_size as usize) };
        let fs = Fat16Fs::new(ram_disk).expect("failed to initialize FAT16 filesystem");

        fs
    });
}

#[inline]
pub fn fs() -> &'static Fat16Fs {
    FILESYSTEM.get().expect("filesystem not initialized")
}

#[derive(Debug)]
pub enum GetFileError {
    InvalidPath,
    NotFound,
    NotADirectory,
}

pub trait File: Read + Write + Seek + Send + Sync {
    fn name(&self) -> &str;
    fn size(&self) -> usize;
}

impl core::fmt::Debug for dyn File {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File")
            .field("name", &self.name())
            .field("size", &self.size())
            .finish()
    }
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
        crate::println!("DirIter::get_entry");
        let path = path.into();
        if let Some((current_dir, children)) = path.enter() {
            for entry in self {
                if entry.name() == current_dir.as_str() {
                    return entry.get_entry(children);
                }
            }
            Err(GetFileError::NotFound)
        } else {
            crate::println!("DirIter::get_entry:no children");
            for entry in self {
                if entry.name() == path.as_str() {
                    return Ok(entry);
                }
            }
            Err(GetFileError::NotFound)
        }
    }
}
