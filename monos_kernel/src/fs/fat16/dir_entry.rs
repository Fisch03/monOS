use super::{Fat16File, Fat16Fs};
use crate::fs::{DirEntry, DirIter};
use crate::utils::BitField;
use alloc::string::String;
use core::{mem, str::FromStr};

#[derive(Debug)]
#[repr(C, packed)]
struct Fat16RawEntry {
    name: [u8; 8],
    extension: [u8; 3],
    attributes: u8,
    _reserved: u8,
    creation_time_tenths: u8,
    creation_time: [u8; 2],
    creation_date: [u8; 2],
    last_access_date: [u8; 2],
    _reserved2: [u8; 2],
    last_write_time: [u8; 2],
    last_write_date: [u8; 2],
    first_cluster: [u8; 2],
    size: [u8; 4],
}

impl Fat16RawEntry {
    pub unsafe fn new(fs: &Fat16Fs, sector: u32, offset: u32) -> Self {
        let mut raw_entry = [0u8; mem::size_of::<Fat16RawEntry>()];
        fs.seek(sector, offset);
        fs.read(&mut raw_entry);
        unsafe { mem::transmute(raw_entry) }
    }
}

#[repr(C, packed)]
struct Fat16LongFileNameEntry {
    sequence_number: u8,
    name1: [u8; 10],
    attributes: u8,
    _reserved: u8,
    checksum: u8,
    name2: [u8; 12],
    _reserved2: u16,
    name3: [u8; 4],
}

impl Fat16LongFileNameEntry {
    pub unsafe fn new(fs: &Fat16Fs, sector: u32, offset: u32) -> Self {
        let mut raw_entry = [0u8; mem::size_of::<Fat16LongFileNameEntry>()];
        fs.seek(sector, offset);
        fs.read(&mut raw_entry);
        mem::transmute(raw_entry)
    }
}

#[derive(Debug, Clone)]
pub struct Fat16DirEntry<'fs> {
    pub(crate) name: String,
    pub(crate) attributes: u8,
    pub(crate) first_cluster: u16,
    pub(crate) size: u32,
    fs: &'fs Fat16Fs,
}

#[derive(Debug)]
pub enum DirEntryError {
    NoMoreEntries,
    FreeEntry,
}

impl<'fs> Fat16DirEntry<'fs> {
    #[inline]
    pub fn first_sector(&self) -> u32 {
        self.fs.first_data_sector
            + (self.first_cluster as u32 - 2) * self.fs.sectors_per_cluster as u32
    }

    pub fn new(fs: &'fs Fat16Fs, sector: u32, offset: u32) -> Result<(Self, usize), DirEntryError> {
        let mut bytes_read = 32;
        let raw_entry = unsafe { Fat16RawEntry::new(fs, sector, offset) };

        let attributes = raw_entry.attributes;
        // crate::println!("attributes: {:08b}", attributes);
        if attributes.get_bits(0..4) == 0x0F {
            // crate::println!("LFN entry");
            // LFN entry
            let lfn_entry =
                unsafe { &*(&raw_entry as *const Fat16RawEntry as *const Fat16LongFileNameEntry) };

            let name = parse_lfn_str(&lfn_entry.name1)
                .chain(parse_lfn_str(&lfn_entry.name2))
                .chain(parse_lfn_str(&lfn_entry.name3));

            if lfn_entry.sequence_number.get_bit(6) {
                // Last LFN entry
                bytes_read += 32;
                let last_entry = unsafe { Fat16RawEntry::new(fs, sector, offset + 32) };

                let name = name.chain(parse_lfn_str(&last_entry.name));
                let name = name.take_while(|&c| c != char::from(0)).collect();

                Self::finalize(fs, name, &last_entry).map(|entry| (entry, bytes_read))
            } else {
                Self::continue_from_lfn(fs, sector, offset + 32, name, bytes_read)
            }
        } else {
            let name = String::from_str(
                core::str::from_utf8(&raw_entry.name)
                    .unwrap()
                    .trim_end_matches(char::from(0)),
            )
            .unwrap();

            Self::finalize(fs, name, &raw_entry).map(|entry| (entry, bytes_read))
        }
    }

    fn continue_from_lfn(
        _fs: &'fs Fat16Fs,
        _sector: u32,
        _offset: u32,
        _name: impl Iterator<Item = char>,
        _bytes_read: usize,
    ) -> Result<(Self, usize), DirEntryError> {
        todo!("lfn entries spanning multiple fat entries")
    }

    fn finalize(
        fs: &'fs Fat16Fs,
        name: String,
        raw_entry: &Fat16RawEntry,
    ) -> Result<Self, DirEntryError> {
        match raw_entry.name[0] {
            0x00 => return Err(DirEntryError::NoMoreEntries),
            0xE5 => return Err(DirEntryError::FreeEntry),
            _ => {}
        }

        let first_cluster = u16::from_le_bytes(raw_entry.first_cluster);
        let size = u32::from_le_bytes(raw_entry.size);
        let attributes = raw_entry.attributes;

        /*
        crate::println!(
            "name: {}, first_cluster: {}, size: {}, attributes: {:08b}",
            name,
            first_cluster,
            size,
            attributes
        );
        */

        Ok(Self {
            name,
            attributes,
            first_cluster,
            size,
            fs,
        })
    }
}

fn parse_lfn_str(data: &[u8]) -> impl Iterator<Item = char> + '_ {
    data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .map(|c| char::from(c as u8))
}

impl<'fs> DirEntry for Fat16DirEntry<'fs> {
    type File = Fat16File<'fs>;
    type DirIter = Fat16DirIter<'fs>;

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn is_dir(&self) -> bool {
        self.attributes.get_bit(4)
    }

    fn iter(&self) -> Option<Self::DirIter> {
        if !self.is_dir() {
            return None;
        }

        Some(Fat16DirIter {
            fs: self.fs,
            sector: self.first_sector(),
            offset: 0,
        })
    }

    fn as_file(&self) -> Option<Self::File> {
        if self.is_dir() {
            return None;
        }

        Some(Fat16File::new(self.fs, self.clone()))
    }
}

#[derive(Clone)]
pub struct Fat16DirIter<'fs> {
    fs: &'fs Fat16Fs,
    sector: u32,
    offset: u32,
}

impl<'fs> Fat16DirIter<'fs> {
    pub fn new(fs: &'fs Fat16Fs, sector: u32) -> Self {
        Self {
            fs,
            sector,
            offset: 0,
        }
    }
}

impl<'fs> DirIter for Fat16DirIter<'fs> {}

impl<'fs> Iterator for Fat16DirIter<'fs> {
    type Item = Fat16DirEntry<'fs>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = Fat16DirEntry::new(self.fs, self.sector, self.offset);
        match entry {
            Ok((entry, bytes_read)) => {
                self.offset += bytes_read as u32;
                Some(entry)
            }
            Err(DirEntryError::NoMoreEntries) => None,
            Err(DirEntryError::FreeEntry) => {
                self.offset += 32;
                self.next()
            }
        }
    }
}
