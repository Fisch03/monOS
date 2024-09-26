pub use monos_std::{
    filesystem::{Path, PathBuf},
    io::{Read, Seek, Write},
};

use alloc::boxed::Box;

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

impl Into<DirEntry> for Box<dyn File> {
    fn into(self) -> DirEntry {
        DirEntry::File(self)
    }
}

pub trait Directory: Send + Sync {
    fn name(&self) -> &str;

    fn iter(&self) -> Box<dyn Iterator<Item = DirEntry>>
    where
        Self: 'static;
    /*
    fn add_file(&mut self, file: Box<dyn File>);
    fn add_directory(&mut self, directory: Box<dyn Directory>);
    */
}

impl core::fmt::Debug for dyn Directory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Directory")
            .field("name", &self.name())
            .finish()
    }
}

impl Into<DirEntry> for Box<dyn Directory> {
    fn into(self) -> DirEntry {
        DirEntry::Directory(self)
    }
}

#[derive(Debug)]
pub enum DirEntry {
    File(Box<dyn File>),
    Directory(Box<dyn Directory>),
}

impl DirEntry {
    pub fn name(&self) -> &str {
        match self {
            Self::File(file) => file.name(),
            Self::Directory(dir) => dir.name(),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Directory(_))
    }

    pub fn as_file(&self) -> Option<&dyn File> {
        match self {
            Self::File(file) => Some(file.as_ref()),
            _ => None,
        }
    }

    pub fn as_dir(&self) -> Option<&dyn Directory> {
        match self {
            Self::Directory(dir) => Some(dir.as_ref()),
            _ => None,
        }
    }
}

pub trait AbstractDirEntry: Sized {
    type File: File;
    type Directory: Directory;
    type DirIter: Iterator<Item = Self> + DirIter<Item = Self>;

    fn name(&self) -> &str;
    fn is_dir(&self) -> bool;

    fn as_file(&self) -> Option<Self::File>;
    fn as_dir(&self) -> Option<Self::Directory>;
    fn as_entry(&self) -> DirEntry
    where
        <Self as AbstractDirEntry>::File: 'static,
        <Self as AbstractDirEntry>::Directory: 'static,
    {
        if self.is_dir() {
            DirEntry::Directory(Box::new(self.as_dir().unwrap()))
        } else {
            DirEntry::File(Box::new(self.as_file().unwrap()))
        }
    }

    fn iter(&self) -> Option<Self::DirIter>;

    fn get_entry<'p, P: Into<Path<'p>>>(&self, path: P) -> Result<Self, GetFileError> {
        let mut iter = self.iter().ok_or(GetFileError::NotADirectory)?;
        iter.get_entry(path)
    }
}

pub trait DirIter: Iterator + Sized
where
    Self::Item: AbstractDirEntry,
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
