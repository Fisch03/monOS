use super::{allocation_table, Fat16Fs, Fat16Node};
use super::{Read, Seek};
use crate::fs::File;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU16, Ordering};

#[derive(Debug)]
pub struct Fat16File {
    first_cluster: u16,
    current_cluster: AtomicU16,
}

impl Fat16File {
    pub fn new(fs: Arc<Fat16Fs>, node: &Fat16Node) -> File {
        let first_cluster = node.first_cluster - 2;

        let data = Self {
            first_cluster,
            current_cluster: first_cluster.into(),
        };

        File::new(node.name.clone(), node.size as usize, fs, data)
    }

    pub fn read(file: &File, fs: &Fat16Fs, buf: &mut [u8]) -> usize {
        let data = file.data::<Self>();

        let cluster_size = fs.cluster_size() as usize;

        let current_cluster = data.current_cluster.load(Ordering::Relaxed);
        let current_pos = file.pos.load(Ordering::Relaxed);
        let cluster = if current_pos % cluster_size == 0 && current_pos != 0 {
            use allocation_table::AllocationType;

            let entry = allocation_table::lookup_allocation(fs, current_cluster);
            match entry {
                AllocationType::Next(next_cluster) => next_cluster,
                _ => return 0,
            }
        } else {
            current_cluster
        };

        let cluster_pos = current_pos % cluster_size;
        let cluster_remaining = cluster_size - cluster_pos;
        let file_remaining = file.size - current_pos as usize;
        let read_size = buf
            .len()
            .min(cluster_remaining as usize)
            .min(file_remaining as usize);

        if read_size == 0 {
            return 0;
        }

        let pos = fs.cluster_offset(cluster as u32) as usize + cluster_pos;
        fs.ramdisk.set_pos(pos as usize);
        let read = fs.ramdisk.read(&mut buf[..read_size]);
        if read == 0 {
            return 0;
        }

        file.pos.fetch_add(read, Ordering::Relaxed);
        data.current_cluster.store(cluster, Ordering::Relaxed);

        read
    }

    pub fn write(file: &mut File, _fs: &Fat16Fs, _buf: &[u8]) -> usize {
        let _data = file.data_mut::<Self>();

        todo!()
    }

    pub fn seek(file: &File, _pos: usize) {
        let _data = file.data::<Self>();

        todo!("update current cluster based on pos");
    }
}
