pub trait Read {
    fn read(&self, buf: &mut [u8]) -> usize;

    fn read_all(&self, buf: &mut [u8]) -> usize {
        let mut total_read = 0;
        while total_read < buf.len() {
            let read = self.read(&mut buf[total_read..]);
            if read == 0 {
                break;
            }
            total_read += read;
        }
        total_read
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> usize;

    fn write_all(&mut self, buf: &[u8]) -> usize {
        let mut total_written = 0;
        while total_written < buf.len() {
            let written = self.write(&buf[total_written..]);
            if written == 0 {
                break;
            }
            total_written += written;
        }
        total_written
    }
}

pub trait Seek {
    fn seek(&self, pos: usize);
}

use core::sync::atomic::{AtomicUsize, Ordering};
pub struct SliceReader<'a> {
    data: &'a [u8],
    pos: AtomicUsize,
}

impl SliceReader<'_> {
    pub fn new(data: &[u8]) -> SliceReader {
        SliceReader {
            data,
            pos: AtomicUsize::new(0),
        }
    }
}

impl Read for SliceReader<'_> {
    fn read(&self, buf: &mut [u8]) -> usize {
        let pos = self.pos.load(Ordering::Relaxed);
        let read = buf.len().min(self.data.len() - pos);
        buf[..read].copy_from_slice(&self.data[pos..pos + read]);
        self.pos.store(pos + read, Ordering::Relaxed);
        read
    }
}

impl Seek for SliceReader<'_> {
    fn seek(&self, pos: usize) {
        self.pos.store(pos, Ordering::Relaxed);
    }
}
