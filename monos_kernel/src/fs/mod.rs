pub mod fat16;
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
    let fs = Fat16Fs::new(ram_disk).expect("failed to initialize FAT16 filesystem");
}
