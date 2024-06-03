use super::{ramdisk::RamDisk, DirEntry, File, Path, Read, Seek, Write};
use core::mem;

mod dir_entry;
use dir_entry::{Fat16DirEntry, Fat16DirIter};

mod file;
use file::Fat16File;

mod allocation_table;

const DIR_ENTRY_SIZE: u32 = 32;

#[derive(Debug)]
pub struct Fat16Fs {
    ramdisk: RamDisk,
    first_root_sector: u32,
    first_fat_sector: u32,
    first_data_sector: u32,
    root_dir_sectors: u32,
    cluster_count: u32,
    bytes_per_sector: u32,
    sectors_per_cluster: u8,
}

#[derive(Debug)]
pub enum Fat16Error {
    NotFAT16,
}

impl Fat16Fs {
    pub fn new(mut ramdisk: RamDisk) -> Result<Self, Fat16Error> {
        let mut bios_parameter_block = [0u8; mem::size_of::<BiosParameterBlock>()];
        ramdisk.seek(mem::offset_of!(BootSector, bios_parameter_block));
        ramdisk.read(&mut bios_parameter_block);

        // safety: we check that the parameter block is valid before using it in any way
        let bios_parameter_block =
            unsafe { &*(bios_parameter_block.as_ptr() as *const BiosParameterBlock) };

        if &bios_parameter_block.fs_type != b"FAT16   " {
            return Err(Fat16Error::NotFAT16);
        }

        Ok(Self {
            ramdisk,
            first_data_sector: bios_parameter_block.first_data_sector(),
            first_fat_sector: bios_parameter_block.first_fat_sector(),
            first_root_sector: bios_parameter_block.first_root_sector(),
            root_dir_sectors: bios_parameter_block.root_dir_sectors(),
            cluster_count: bios_parameter_block.cluster_count(),
            bytes_per_sector: bios_parameter_block.bytes_per_sector(),
            sectors_per_cluster: bios_parameter_block.sectors_per_cluster,
        })
    }

    #[inline]
    fn sector_offset(&self, sector: u32) -> u32 {
        sector * self.bytes_per_sector
    }

    #[inline]
    fn cluster_size(&self) -> u32 {
        self.bytes_per_sector * self.sectors_per_cluster as u32
    }

    #[inline]
    pub fn seek(&self, sector: u32, offset: u32) {
        self.ramdisk
            .seek((self.sector_offset(sector) + offset) as usize);
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) {
        self.ramdisk.read(buf);
    }

    pub fn iter_root_dir(&mut self) -> Fat16DirIter<'_> {
        Fat16DirIter::new(self, self.first_root_sector)
    }
}

#[repr(C, packed)]
struct BootSector {
    code: [u8; 3],
    os_name: [u8; 8],
    bios_parameter_block: BiosParameterBlock,
    boot_code: [u8; 448],
    bootable_signature: u16,
}

#[derive(Debug)]
#[repr(C, packed)]
struct BiosParameterBlock {
    bytes_per_sector: [u8; 2],
    sectors_per_cluster: u8,
    reserved_sectors: [u8; 2],
    fat_count: u8,
    root_dir_entries: [u8; 2],
    total_sectors_small: [u8; 2],
    media_descriptor: u8,
    sectors_per_fat: [u8; 2],
    sectors_per_track: [u8; 2],
    heads: [u8; 2],
    hidden_sectors: [u8; 4],
    total_sectors_large: [u8; 4],
    drive_number: u8,
    _reserved: u8,
    extended_boot_signature: u8,
    volume_id: [u8; 4],
    volume_label: [u8; 11],
    fs_type: [u8; 8],
}

impl BiosParameterBlock {
    #[inline]
    fn root_dir_sectors(&self) -> u32 {
        let root_dir_bytes = u16::from_le_bytes(self.root_dir_entries) as u32 * DIR_ENTRY_SIZE;
        let bytes_per_sector = u16::from_le_bytes(self.bytes_per_sector) as u32;
        (root_dir_bytes + bytes_per_sector - 1) / bytes_per_sector
    }

    #[inline]
    fn first_root_sector(&self) -> u32 {
        u16::from_le_bytes(self.reserved_sectors) as u32
            + self.fat_count as u32 * u16::from_le_bytes(self.sectors_per_fat) as u32
    }

    #[inline]
    fn first_fat_sector(&self) -> u32 {
        u16::from_le_bytes(self.reserved_sectors) as u32
    }

    #[inline]
    fn first_data_sector(&self) -> u32 {
        self.first_root_sector() + self.root_dir_sectors()
    }

    #[inline]
    fn total_sectors(&self) -> u32 {
        let total_sectors_small = u16::from_le_bytes(self.total_sectors_small);
        if total_sectors_small != 0 {
            total_sectors_small as u32
        } else {
            u32::from_le_bytes(self.total_sectors_large)
        }
    }

    #[inline]
    fn cluster_count(&self) -> u32 {
        (self.total_sectors() - self.first_data_sector()) / self.sectors_per_cluster as u32
    }

    #[inline]
    fn bytes_per_sector(&self) -> u32 {
        u16::from_le_bytes(self.bytes_per_sector) as u32
    }
}
