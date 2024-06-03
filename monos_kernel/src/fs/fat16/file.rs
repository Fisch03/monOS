use super::{allocation_table, Fat16DirEntry, Fat16Fs};
use super::{DirEntry, File, Path, Read, Seek, Write};
use core::mem;

pub struct Fat16File<'fs> {
    fs: &'fs Fat16Fs,
    dir_entry: Fat16DirEntry<'fs>,
    pos: u32,
}

impl<'fs> Fat16File<'fs> {
    pub fn new(fs: &'fs Fat16Fs, dir_entry: Fat16DirEntry<'fs>) -> Self {
        Self {
            fs,
            dir_entry,
            pos: 0,
        }
    }
}

impl<'fs> File for Fat16File<'fs> {
    fn name(&self) -> &str {
        &self.dir_entry.name
    }

    fn size(&self) -> usize {
        self.dir_entry.size as usize
    }
}

impl<'fs> Read for Fat16File<'fs> {
    fn read(&self, buf: &mut [u8]) -> usize {
        let mut bytes_read = 0;
        let mut buf_pos = 0;

        unimplemented!()
    }
}

impl<'fs> Write for Fat16File<'fs> {
    fn write(&mut self, buf: &[u8]) -> usize {
        unimplemented!()
    }
}

impl<'fs> Seek for Fat16File<'fs> {
    fn seek(&self, pos: usize) {
        unimplemented!()
    }
}
