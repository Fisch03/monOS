use super::ramdisk::RamDisk;
use core::mem;

pub struct Fat16Fs {
    ramdisk: RamDisk,
}

#[derive(Debug)]
pub enum Fat16Error {
    NotFAT16,
}

impl Fat16Fs {
    pub fn new(ramdisk: RamDisk) -> Result<Self, Fat16Error> {
        let mut fs_type = [0u8; 8];
        ramdisk.read(mem::offset_of!(BootSector, fs_type), &mut fs_type);
        if fs_type != *b"FAT16   " {
            return Err(Fat16Error::NotFAT16);
        }

        Ok(Self { ramdisk })
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct BootSector {
    code: [u8; 3],
    os_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    root_dir_entries: u16,
    total_sectors_small: u16,
    media_descriptor: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_sectors_large: u32,
    drive_number: u8,
    _reserved: u8,
    extended_boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    boot_code: [u8; 448],
    bootable_signature: u16,
}
