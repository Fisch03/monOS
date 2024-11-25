use super::{ramdisk::RamDisk, *};
use alloc::{boxed::Box, sync::Arc};
use core::mem;

mod node;
use node::{Fat16DirIter, Fat16Node};

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
    // root_dir_sectors: u32,
    // cluster_count: u32,
    bytes_per_sector: u32,
    sectors_per_cluster: u8,
}

#[derive(Debug)]
pub enum Fat16Error {
    NotFAT16,
}

impl Fat16Fs {
    pub fn new(ramdisk: RamDisk) -> Result<Self, Fat16Error> {
        let mut bios_parameter_block = [0u8; mem::size_of::<BiosParameterBlock>()];
        ramdisk.set_pos(mem::offset_of!(BootSector, bios_parameter_block));
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
            // root_dir_sectors: bios_parameter_block.root_dir_sectors(),
            // cluster_count: bios_parameter_block.cluster_count(),
            bytes_per_sector: bios_parameter_block.bytes_per_sector(),
            sectors_per_cluster: bios_parameter_block.sectors_per_cluster,
        })
    }

    #[inline]
    fn sector_offset(&self, sector: u32) -> u32 {
        sector * self.bytes_per_sector
    }

    #[inline]
    fn cluster_offset(&self, cluster: u32) -> u32 {
        self.sector_offset(self.first_data_sector + cluster * self.sectors_per_cluster as u32)
    }

    #[inline]
    fn cluster_size(&self) -> u32 {
        self.bytes_per_sector * self.sectors_per_cluster as u32
    }

    #[inline]
    fn seek(&self, sector: u32, offset: u32) {
        self.ramdisk
            .set_pos((self.sector_offset(sector) + offset) as usize);
    }

    #[inline]
    fn read(&self, buf: &mut [u8]) {
        self.ramdisk.read(buf);
    }

    pub fn iter_root_dir(&self) -> Fat16DirIter {
        Fat16DirIter::new(self, self.first_root_sector)
    }
}

enum Fat16NodeData {
    RootDir,
    Node(Fat16Node),
}

impl FileSystem for Fat16Fs {
    fn open(self: Arc<Self>, node: &VFSNode) -> Result<File, OpenError> {
        let fs = node.fs();
        let node: &Fat16NodeData = fs.as_ref().unwrap().data();

        let node = match node {
            Fat16NodeData::Node(node) => node,
            _ => return Err(OpenError::NotAFile),
        };

        Ok(Fat16File::new(self, node))
    }

    fn close(&self, _file: File) -> Result<(), CloseError> {
        //TODO: maybe do something here
        Ok(())
    }

    fn list(self: Arc<Fat16Fs>, node: Arc<VFSNode>) {
        let fs = node.fs();
        let node_data: &Fat16NodeData = fs.as_ref().unwrap().data();
        let iter = match node_data {
            Fat16NodeData::RootDir => self.iter_root_dir(),
            Fat16NodeData::Node(node) => {
                if node.is_dir() {
                    node.iter(&self)
                } else {
                    return;
                }
            }
        };

        for child in iter {
            let node_type = if child.is_dir() {
                VFSNodeType::Directory
            } else {
                VFSNodeType::File {
                    size: child.size as usize,
                }
            };
            VFSNode::add_child(
                &node,
                child.name.clone(),
                node_type,
                Some(FSData {
                    fs: self.clone(),
                    data: Box::new(Fat16NodeData::Node(child)),
                }),
            );
        }
    }

    fn read(&self, file: &File, buf: &mut [u8]) -> usize {
        Fat16File::read(file, self, buf)
    }
    fn write(&self, file: &mut File, buf: &[u8]) -> usize {
        Fat16File::write(file, self, buf)
    }
    fn seek(&self, file: &File, pos: usize) {
        Fat16File::seek(file, pos)
    }

    fn mount(self, node: &VFSNode) {
        node.set_fs(FSData::new(self, Fat16NodeData::RootDir));
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
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

    // #[inline]
    // fn total_sectors(&self) -> u32 {
    //     let total_sectors_small = u16::from_le_bytes(self.total_sectors_small);
    //     if total_sectors_small != 0 {
    //         total_sectors_small as u32
    //     } else {
    //         u32::from_le_bytes(self.total_sectors_large)
    //     }
    // }

    // #[inline]
    // fn cluster_count(&self) -> u32 {
    //     (self.total_sectors() - self.first_data_sector()) / self.sectors_per_cluster as u32
    // }

    #[inline]
    fn bytes_per_sector(&self) -> u32 {
        u16::from_le_bytes(self.bytes_per_sector) as u32
    }
}
