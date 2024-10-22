mod address;
pub use address::{PhysicalAddress, VirtualAddress};

mod paging;
pub use paging::*;

mod alloc_heap;

use bootloader_api::info::BootInfo;
use core::arch::asm;

use spin::Once;
static PHYSICAL_MEM_OFFSET: Once<u64> = Once::new();
pub fn physical_mem_offset() -> VirtualAddress {
    VirtualAddress::new(*PHYSICAL_MEM_OFFSET.get().unwrap())
}

pub unsafe fn init(boot_info: &BootInfo) {
    let phys_mem_offset = boot_info.physical_memory_offset.as_ref().unwrap();
    let phys_mem_offset = VirtualAddress::new(*phys_mem_offset);
    PHYSICAL_MEM_OFFSET.call_once(|| phys_mem_offset.as_u64());

    paging::init(phys_mem_offset, boot_info);

    alloc_heap::init();
}

#[derive(Debug, Clone)]
#[repr(C, packed(2))]
pub struct DTPointer {
    pub limit: u16,
    pub base: VirtualAddress,
}

impl DTPointer {
    /// load the IDT at the pointer adress.
    ///
    /// safety: the pointer must point to a valid IDT and have the correct limit.
    pub unsafe fn load_idt(&self) {
        asm!("lidt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }

    /// load the GDT at the pointer adress.
    ///
    /// safety: the pointer must point to a valid GDT and have the correct limit.
    pub unsafe fn load_gdt(&self) {
        asm!("lgdt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }
}
