use super::{allocation_table, Fat16DirEntry, Fat16Fs};
use super::{File, Read, Seek, Write};
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};

pub struct Fat16File<'fs> {
    fs: &'fs Fat16Fs,
    dir_entry: Fat16DirEntry<'fs>,
    current_cluster: AtomicU16,
    pos: AtomicU32,
}

impl<'fs> Fat16File<'fs> {
    pub fn new(fs: &'fs Fat16Fs, dir_entry: Fat16DirEntry<'fs>) -> Self {
        let first_cluster = dir_entry.first_cluster - 2;

        Self {
            fs,
            dir_entry,
            current_cluster: first_cluster.into(),
            pos: 0.into(),
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
        let cluster_size = self.fs.cluster_size();

        let current_cluster = self.current_cluster.load(Ordering::Relaxed);
        let current_pos = self.pos.load(Ordering::Relaxed);
        let cluster = if current_pos % cluster_size == 0 && current_pos != 0 {
            use allocation_table::AllocationType;

            let entry = allocation_table::lookup_allocation(self.fs, current_cluster);
            match entry {
                AllocationType::Next(next_cluster) => next_cluster,
                _ => return 0,
            }
        } else {
            current_cluster
        };

        let cluster_pos = current_pos % cluster_size;
        let cluster_remaining = cluster_size - cluster_pos;
        let file_remaining = self.dir_entry.size - current_pos;
        let read_size = buf
            .len()
            .min(cluster_remaining as usize)
            .min(file_remaining as usize);

        if read_size == 0 {
            return 0;
        }

        let pos = self.fs.cluster_offset(cluster as u32) + cluster_pos;
        self.fs.ramdisk.seek(pos as usize);
        let read = self.fs.ramdisk.read(&mut buf[..read_size]);
        if read == 0 {
            return 0;
        }

        self.pos.fetch_add(read as u32, Ordering::Relaxed);
        self.current_cluster.store(cluster, Ordering::Relaxed);

        read
    }
}

impl<'fs> Write for Fat16File<'fs> {
    fn write(&mut self, _buf: &[u8]) -> usize {
        todo!()
    }
}

impl<'fs> Seek for Fat16File<'fs> {
    fn seek(&self, _pos: usize) {
        todo!()
    }
}
