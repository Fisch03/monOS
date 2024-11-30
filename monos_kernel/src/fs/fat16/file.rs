use super::{allocation_table, Fat16Fs, Fat16Node};
use super::{Read, Seek};
use crate::fs::File;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicI32, Ordering};

#[derive(Debug)]
struct AtomicCurrentCluster(AtomicI32);

impl AtomicCurrentCluster {
    fn new(val: Option<i16>) -> Self {
        Self(AtomicI32::new(val.map(|v| v as i32).unwrap_or(-1)))
    }

    fn load(&self, order: Ordering) -> Option<u16> {
        let val = self.0.load(order);
        if val < 0 {
            None
        } else {
            Some(val as u16)
        }
    }

    fn store(&self, val: Option<u16>, order: Ordering) {
        let val = val.map(|v| v as i32).unwrap_or(-1);
        self.0.store(val, order);
    }
}

#[derive(Debug)]
pub struct Fat16File {
    first_cluster: u16,
    current_cluster: AtomicCurrentCluster,
}

impl Fat16File {
    pub fn new(fs: Arc<Fat16Fs>, node: &Fat16Node) -> File {
        let first_cluster = node.first_cluster;

        let data = Self {
            first_cluster,
            current_cluster: AtomicCurrentCluster::new(None),
        };

        File::new(node.name.clone(), node.size as usize, fs, data)
    }

    pub fn read(file: &File, fs: &Fat16Fs, buf: &mut [u8]) -> usize {
        let data = file.data::<Self>();

        let cluster_size = fs.cluster_size() as usize;

        let current_cluster = data.current_cluster.load(Ordering::Relaxed);
        let current_pos = file.pos.load(Ordering::Relaxed);
        let cluster = match current_cluster {
            None => data.first_cluster,
            Some(current_cluster) => {
                if current_pos % cluster_size == 0 {
                    use allocation_table::AllocationType;

                    let entry = allocation_table::lookup_allocation(fs, current_cluster);
                    match entry {
                        AllocationType::Next(next_cluster) => next_cluster,
                        _ => return 0,
                    }
                } else {
                    current_cluster
                }
            }
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
        data.current_cluster.store(Some(cluster), Ordering::Relaxed);

        read
    }

    pub fn write(file: &mut File, _fs: &Fat16Fs, _buf: &[u8]) -> usize {
        let _data = file.data_mut::<Self>();

        todo!()
    }

    pub fn seek(file: &File, mut pos: usize) {
        let data = file.data::<Self>();
        let fs: &Fat16Fs = file.fs().as_any().downcast_ref::<Fat16Fs>().unwrap();

        let current_pos = file.pos.load(Ordering::Relaxed);

        if pos > file.size {
            pos = file.size;
        }

        if pos == current_pos {
            return;
        }

        let current_cluster = data.current_cluster.load(Ordering::Relaxed);

        let new_offset_in_clusters = fs.clusters_from_bytes(pos as u32);
        let old_offset_in_clusters = fs.clusters_from_bytes(current_pos as u32);

        let new_cluster = if pos == 0 {
            None
        } else if new_offset_in_clusters == old_offset_in_clusters {
            current_cluster
        } else {
            use allocation_table::AllocationType;

            let mut cluster = data.first_cluster;
            let clusters_to_seek = new_offset_in_clusters - 1 + data.first_cluster as u32;

            for c in data.first_cluster as u32..clusters_to_seek {
                match allocation_table::lookup_allocation(fs, cluster) {
                    AllocationType::Next(next_cluster) => {
                        cluster = next_cluster;
                    }
                    AllocationType::EndOfFile => {
                        pos = fs.bytes_from_clusters(c + 1) as usize;

                        break;
                    }
                    _ => panic!("invalid data in FAT16 fs!"),
                }
            }
            Some(cluster)
        };

        data.current_cluster.store(new_cluster, Ordering::Relaxed);
        file.pos.store(pos, Ordering::Relaxed);
    }
}
