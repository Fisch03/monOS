pub use monos_std::{
    fs::{ArrayPath, FileHandle, Path, PathBuf},
    io::{Read, Seek, Write},
};

pub mod vfs;
pub use vfs::*;

pub mod fat16;
use fat16::Fat16Fs;

mod ramdisk;
use ramdisk::RamDisk;

use crate::mem::VirtualAddress;
use bootloader_api::BootInfo;

use spin::Once;

// TODO: the fs impl currently has no safety at all in regards to opening the same file multiple times. that should definitely be added at some point but for now we ball

static FS_ROOT_NODE: Once<VFS> = Once::new();

pub fn init(boot_info: &BootInfo) {
    FS_ROOT_NODE.call_once(|| {
        let fs = VFS::new();

        let ramdisk_start = VirtualAddress::new(
            boot_info
                .ramdisk_addr
                .into_option()
                .expect("no ramdisk found"),
        );
        let ramdisk_size = boot_info.ramdisk_len;

        let ram_disk = unsafe { RamDisk::new(ramdisk_start, ramdisk_size as usize) };
        fs.mount(Fat16Fs::new(ram_disk).expect("failed to initialize FAT16 filesystem"))
            .unwrap();

        fs
    });
}

#[inline]
pub fn fs() -> &'static VFS {
    FS_ROOT_NODE.get().expect("filesystem not initialized")
}
