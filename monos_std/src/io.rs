use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum SeekMode {
    Start = 0,
    End = 1,
    Current = 2,
}

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
    fn set_pos(&self, pos: usize);
    fn get_pos(&self) -> usize;

    fn seek(&self, offset: i64, mode: SeekMode) -> usize {
        let new_pos = match mode {
            SeekMode::Start => offset,
            SeekMode::End => (self.get_pos() as i64).saturating_add(offset),
            SeekMode::Current => (self.get_pos() as i64).saturating_add(offset),
        };
        self.set_pos(new_pos as usize);
        new_pos as usize
    }
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
    fn set_pos(&self, pos: usize) {
        self.pos.store(pos, Ordering::Relaxed);
    }

    fn get_pos(&self) -> usize {
        self.pos.load(Ordering::Relaxed)
    }
}
