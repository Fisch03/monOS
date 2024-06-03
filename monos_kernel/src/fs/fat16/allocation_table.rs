use super::{Fat16Fs, Read, Seek};

pub enum AllocationType {
    Free,
    NotAllowed,
    Next(u16),
    Bad,
    EndOfFile,
}

pub fn lookup_allocation(fs: &Fat16Fs, cluster: u16) -> AllocationType {
    let fat_offset = cluster as u32 * 2;
    let fat_sector = fs.first_fat_sector + (fat_offset / fs.bytes_per_sector);
    let fat_offset = fat_offset % fs.bytes_per_sector;

    let mut fat_entry = [0u8; 2];
    fs.ramdisk
        .seek(fs.sector_offset(fat_sector) as usize + fat_offset as usize);
    fs.ramdisk.read(&mut fat_entry);

    let fat_entry = u16::from_le_bytes(fat_entry);
    match fat_entry {
        0x0000 => AllocationType::Free,
        0x0001 | 0x0002 => AllocationType::NotAllowed,
        0xFFF7 => AllocationType::Bad,
        0xFFF8..=0xFFFF => AllocationType::EndOfFile,
        _ => AllocationType::Next(fat_entry),
    }
}
