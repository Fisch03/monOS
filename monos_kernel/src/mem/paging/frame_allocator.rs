use crate::utils::BitArray;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::mem::PhysicalAddress;

use super::{Frame, PageSize4K};

pub struct FrameAllocator {
    free_mem: u64,
    total_mem: u64,
    map: BitArray<32768>, // enough for 1 GiB of memory
    start: PhysicalAddress,
    last_allocated: usize,
}

impl FrameAllocator {
    pub fn free_memory(&self) -> u64 {
        self.free_mem
    }

    pub fn used_memory(&self) -> u64 {
        self.total_mem - self.free_mem
    }

    pub fn total_memory(&self) -> u64 {
        self.total_mem
    }

    pub fn new(mem_map: &MemoryRegions, start_frame: Frame) -> Self {
        let start_frame_number = start_frame.number() as usize;

        let mut map = BitArray::new(true);
        let map_end = start_frame.start_address().as_u64() + map.len() as u64 * 4096;

        let mut free = 0;

        for region in mem_map
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .filter(|r| r.start <= map_end)
            .filter(|r| r.end >= start_frame.start_address().as_u64())
        {
            let start = PhysicalAddress::new(region.start);
            // align the end address to the next page
            let end = PhysicalAddress::new(region.end).align(4096);

            free += end.as_u64() - start.as_u64();

            let mut curr: Frame<PageSize4K> = Frame::around(start);
            while curr.start_address().as_u64() < end.as_u64() {
                let curr_num = curr.number() as usize;
                if curr_num >= start_frame_number {
                    map.set(curr_num - start_frame_number, false);
                }
                curr = Frame::new(curr.end_address()).unwrap();
            }
        }

        let total_mem = mem_map.iter().map(|r| r.end - r.start).sum();

        Self {
            free_mem: free,
            total_mem,
            map,
            start: start_frame.start_address(),
            last_allocated: 0,
        }
    }

    pub fn allocate_frame(&mut self, _reason: &str) -> Option<Frame> {
        let frame = if let Some(position) = self.map.iter_from(self.last_allocated).position(|b| !b)
        {
            position + self.last_allocated
        } else {
            self.map.iter().position(|b| !b)?
        };
        self.map.set(frame, true);
        self.last_allocated = frame + 1;

        let frame_addr = self.start + (frame as u64 * 4096);

        // crate::println!(
        //     "allocated {:#x} - {:#x} for {}",
        //     frame_addr.as_u64(),
        //     frame_addr.as_u64() + 4096,
        //     reason
        // );

        self.free_mem -= 4096;

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

            crate::println!(
                "allocated {:#x} - {:#x}",
                frame_addr.as_u64(),
                frame_addr.as_u64() + count as u64 * 4096
            );

            self.free_mem -= count as u64 * 4096;

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

mod test {
    use monos_test::kernel_test;

    #[kernel_test]
    fn test_frame_alloc_all(boot_info: &bootloader_api::BootInfo) -> bool {
        use crate::mem::{Frame, PhysicalAddress};
        use bootloader_api::info::*;

        let start_frame = Frame::around(PhysicalAddress::new(0x0));
        let mut allocator = super::FrameAllocator::new(&boot_info.memory_regions, start_frame);

        crate::println!("unusable regions:");
        for region in boot_info
            .memory_regions
            .iter()
            .filter(|r| r.kind != MemoryRegionKind::Usable)
        {
            crate::println!(
                "region: {:#x} - {:#x} {:?}",
                region.start,
                region.end,
                region.kind
            );
        }

        let mut unusable_regions: [(u64, u64); 100] = [(0, 0); 100];
        let mut unusable_count = 0;
        for region in boot_info.memory_regions.iter() {
            if region.kind != MemoryRegionKind::Usable {
                unusable_regions[unusable_count] = (region.start, region.end);
                unusable_count += 1;
            }
        }

        let mut count = 0;
        while let Some(frame) = allocator.allocate_frame("test") {
            let frame_start = frame.start_address().as_u64();
            let frame_end = frame.end_address().as_u64();

            if count % 100 == 0 {
                crate::print!(
                    "allocated frame no. {} at {:#x} - {:#x}\x1b[0K\r",
                    count,
                    frame_start,
                    frame_end
                );
            }

            for (region_start, region_end) in unusable_regions.iter().take(unusable_count) {
                if frame_start >= *region_start && frame.end_address().as_u64() <= *region_end {
                    crate::println!(
                        "frame {:#x} - {:#x} overlaps with unusable region {:#x} - {:#x}",
                        frame.start_address().as_u64(),
                        frame.end_address().as_u64(),
                        *region_start,
                        *region_end
                    );

                    return false;
                }
            }
            count += 1;
        }
        crate::print!("\n");

        crate::println!("successfully allocated {} frames", count);

        true
    }

    #[kernel_test]
    fn test_frame_write_all(boot_info: &bootloader_api::BootInfo) -> bool {
        use crate::mem::{Frame, PhysicalAddress, VirtualAddress};

        let start_frame = Frame::around(PhysicalAddress::new(0x0));
        let mut allocator = super::FrameAllocator::new(&boot_info.memory_regions, start_frame);

        let phys_mem_offset = boot_info.physical_memory_offset.as_ref().unwrap();
        let phys_mem_offset = VirtualAddress::new(*phys_mem_offset);

        let mut print_div = 0;

        while let Some(frame) = allocator.allocate_frame("test") {
            let frame_start = frame.start_address().as_u64();
            let frame_end = frame.end_address().as_u64();
            let frame_virt = phys_mem_offset + frame_start;

            print_div += 1;
            if print_div % 100 == 0 {
                crate::print!(
                    "writing to frame at {:#x} - {:#x}\x1b[0K...\r",
                    frame_start,
                    frame_end
                );
            }

            let frame_ptr = frame_virt.as_mut_ptr::<u8>();
            unsafe { core::ptr::write_bytes(frame_ptr, 0x00, 4096) };
            for i in 0..4096 {
                if unsafe { *frame_ptr.add(i) } != 0x00 {
                    crate::println!(
                        "frame at {:#x} - {:#x} was not written correctly",
                        frame_start,
                        frame_end
                    );
                    return false;
                }
            }
        }

        crate::print!("\n");
        crate::println!("successfully wrote to all allocated frames");
        true
    }
}
