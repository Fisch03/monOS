use crate::utils::BitArray;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::mem::PhysicalAddress;

use super::{Frame, PageSize4K};

pub struct FrameAllocator {
    map: BitArray<4096>, // enough for 128 MiB of memory
    start: PhysicalAddress,
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
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let frame = self.map.iter().position(|b| !b)?;
        self.map.set(frame, true);
        let frame_addr = self.start + (frame as u64 * 4096);
        Some(Frame::new(frame_addr).unwrap())
    }
}
