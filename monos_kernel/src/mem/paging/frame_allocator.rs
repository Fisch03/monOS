use crate::utils::BitArray;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::mem::PhysicalAddress;

use super::{Frame, PageSize4K};

//TODO: at 8mb allocated frames, the system panics, is there some region that is not being marked as unusable?

pub struct FrameAllocator {
    map: BitArray<32768>, // enough for 1 GiB of memory
    start: PhysicalAddress,
    last_allocated: usize,
}

impl FrameAllocator {
    pub fn new(mem_map: &MemoryRegions, start_frame: Frame) -> Self {
        let start_frame_number = start_frame.number() as usize;

        let mut map = BitArray::new();
        let map_end = start_frame.start_address().as_u64() + map.len() as u64 * 4096;

        for region in mem_map
            .iter()
            .filter(|r| r.kind != MemoryRegionKind::Usable)
            .filter(|r| r.start <= map_end)
            .filter(|r| r.end >= start_frame.start_address().as_u64())
        {
            let start = PhysicalAddress::new(region.start);
            // align the end address to the next page
            let end = PhysicalAddress::new(region.end + 4096).align(4096);

            let mut curr: Frame<PageSize4K> = Frame::around(start);
            while curr.start_address().as_u64() <= end.as_u64() {
                let curr_num = curr.number() as usize;
                if curr_num >= start_frame_number {
                    map.set(curr_num - start_frame_number, true);
                }
                curr = Frame::new(curr.end_address()).unwrap();
            }
        }

        Self {
            map,
            start: start_frame.start_address(),
            last_allocated: 0,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let frame = if let Some(position) = self.map.iter_from(self.last_allocated).position(|b| !b)
        {
            position + self.last_allocated
        } else {
            self.map.iter().position(|b| !b)?
        };
        self.map.set(frame, true);
        self.last_allocated = frame + 1;

        let frame_addr = self.start + (frame as u64 * 4096);

        Some(Frame::new(frame_addr).unwrap())
    }

    pub fn allocate_consecutive(&mut self, amount: usize) -> Option<Frame> {
        let mut start = None;
        let mut count = 0;
        for (i, b) in self.map.iter().enumerate() {
            if !b {
                if start.is_none() {
                    start = Some(i);
                }
                count += 1;
            } else {
                start = None;
                count = 0;
            }

            if count == amount {
                break;
            }
        }

        if let Some(start) = start {
            for i in start..start + count {
                self.map.set(i, true);
            }

            let frame_addr = self.start + (start as u64 * 4096);

            Some(Frame::new(frame_addr).unwrap())
        } else {
            None
        }
    }

    // pub fn reserve_range(&mut self, start: PhysicalAddress, size: usize) {
    //     let mut curr = Frame::around(start);
    //     let end = PhysicalAddress::new(start.as_u64() + size as u64 + 4096).align(4096);
    //     while curr.start_address().as_u64() <= end.as_u64() {
    //         let curr_num = curr.number() as usize;
    //         if curr_num >= self.start_number {
    //             self.map.set(curr_num - self.start_number, true);
    //         }
    //         curr = Frame::new(curr.end_address()).unwrap();
    //     }
    // }
}
