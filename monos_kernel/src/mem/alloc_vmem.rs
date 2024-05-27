use super::VirtualAddress;
use bootloader_api::info::{BootInfo, MemoryRegionKind};

use spin::{Mutex, Once};

pub static VMEM_ALLOCATOR: Once<Mutex<VirtualMemoryAllocator>> = Once::new();

// just a simple bump allocator.
//
// since we are in 64 bit mode, we have almost 256TB of virtual address space for the kernel available so its whatever
pub struct VirtualMemoryAllocator {
    next: VirtualAddress,
}

impl VirtualMemoryAllocator {
    pub fn allocate(&mut self, size: u64) -> VirtualAddress {
        let addr = self.next;
        self.next += size;
        addr
    }
}

pub fn init(physical_memory_offset: VirtualAddress, boot_info: &BootInfo) {
    let mut last_used = physical_memory_offset + boot_info.kernel_addr + boot_info.kernel_len;

    // make sure we don't allocate over anything important
    for region in boot_info
        .memory_regions
        .iter()
        .filter(|region| region.kind != MemoryRegionKind::Usable)
    {
        last_used = last_used.max(physical_memory_offset + region.end);
    }

    VMEM_ALLOCATOR.call_once(|| {
        let allocator = VirtualMemoryAllocator {
            next: last_used.align_up(4096),
        };
        Mutex::new(allocator)
    });
}

/// allocate a chunk of virtual memory
///
/// safety: the caller must ensure to stay within the bounds of the requested memory
pub unsafe fn alloc_vmem(size: u64) -> VirtualAddress {
    VMEM_ALLOCATOR
        .get()
        .expect("memory hasn't been initialized yet")
        .lock()
        .allocate(size)
}
