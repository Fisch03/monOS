use super::Fat16Fs;
use super::{DirEntry, File, Path, Read, Seek, Write};
use core::mem;

pub struct Fat16File<'fs> {
    fs: &'fs Fat16Fs,
    first_cluster: u32,
    size: u32,
    pos: u32,
}

impl<'fs> File for Fat16File<'fs> {
    fn name(&self) -> alloc::string::String {
        unimplemented!()
    }

    fn size(&self) -> usize {
        self.size as usize
    }
}

impl<'fs> Read for Fat16File<'fs> {
    fn read(&self, buf: &mut [u8]) -> usize {
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
