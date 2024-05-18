#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
extern crate alloc;

mod arch;
mod gdt;
pub mod gfx;
pub mod interrupts;
mod mem;
pub mod serial;
mod utils;

use bootloader_api::BootInfo;
use mem::VirtualAddress;

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    interrupts::init();

    let phys_mem_offset = boot_info.physical_memory_offset.as_ref().unwrap();
    let phys_mem_offset = VirtualAddress::new(*phys_mem_offset);

    // safety: the physical memory offset is valid since it was provided by the bootloader.
    // the bootloader config guarantees that the entire physical memory is mapped.
    unsafe { mem::init(phys_mem_offset, &boot_info.memory_regions) };

    let fb = boot_info.framebuffer.take().unwrap();
    gfx::init(fb);
}
