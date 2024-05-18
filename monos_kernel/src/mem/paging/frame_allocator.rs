use alloc::vec::Vec;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use crate::mem::PhysicalAddress;

use super::Frame;

unsafe trait FrameAllocatorTrait {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, _frame: Frame);
}

// initial allocator stage for when no heap is available
struct BumpAllocator {
    regions: &'static MemoryRegions,
    next: usize,
}
impl BumpAllocator {
    fn new(regions: &'static MemoryRegions) -> Self {
        Self { regions, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = Frame> {
        self.regions
            .iter()
            .filter(move |region| region.kind == MemoryRegionKind::Usable)
            .flat_map(|region| region.start as usize..region.end as usize)
            .step_by(4096)
            .map(|addr| Frame::around(PhysicalAddress::new(addr as u64)))
    }

    fn unused_frames_only(&self) -> impl Iterator<Item = Frame> {
        self.usable_frames().skip(self.next)
    }
}
unsafe impl FrameAllocatorTrait for BumpAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        panic!("deallocate_frame called on BumpAllocator")
    }
}

// main allocator stage that needs a heap
struct QueueAllocator {
    queue: Vec<Frame>,
}
unsafe impl FrameAllocatorTrait for QueueAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        self.queue.pop()
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        self.queue.push(frame)
    }
}

impl From<&BumpAllocator> for QueueAllocator {
    fn from(bump: &BumpAllocator) -> Self {
        Self {
            queue: bump.unused_frames_only().collect(),
        }
    }
}

enum AllocatorStage {
    Bump(BumpAllocator),
    Queue(QueueAllocator),
}

impl AllocatorStage {
    // upgrade to the queue stage
    //
    // safety: there must be a heap allocator available
    unsafe fn upgrade(&mut self) {
        match self {
            AllocatorStage::Bump(ref bump) => {
                let queue = QueueAllocator::from(bump);
                *self = AllocatorStage::Queue(queue);
            }
            AllocatorStage::Queue(_) => {}
        }
    }

    fn allocate_frame(&mut self) -> Option<Frame> {
        match self {
            AllocatorStage::Bump(bump) => bump.allocate_frame(),
            AllocatorStage::Queue(queue) => queue.allocate_frame(),
        }
    }
}

pub struct FrameAllocator {
    stage: AllocatorStage,
}

impl FrameAllocator {
    pub fn new(mem_map: &'static MemoryRegions) -> Self {
        let bump = BumpAllocator::new(mem_map);
        let stage = AllocatorStage::Bump(bump);
        Self { stage }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        self.stage.allocate_frame()
    }
}
