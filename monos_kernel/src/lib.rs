#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod gdt;
pub mod gfx;
pub mod interrupts;
mod mem;
pub mod serial;
mod utils;

use bootloader_api::BootInfo;
use mem::VirtualAddress;

pub fn kernel_init(boot_info: &BootInfo) {
    gdt::init();
    interrupts::init();

    let raw_fb_addr = VirtualAddress::from_ptr(boot_info.framebuffer.as_ref().unwrap());

    let phys_mem_offset = boot_info.physical_memory_offset.as_ref().unwrap();
    let phys_mem_offset = VirtualAddress::new(*phys_mem_offset);
    // safety: the physical memory offset is valid since it was provided by the bootloader.
    // the bootloader config guarantees that the entire physical memory is mapped.
    let mapper = unsafe { mem::paging::init(phys_mem_offset) };

    let addresses = [phys_mem_offset.as_u64(), raw_fb_addr.as_u64()];

    for &address in &addresses {
        let virt = VirtualAddress::new(address);
        let phys = mapper.translate_addr(virt);
        dbg!(virt);
        dbg!(phys);

        dbg!("---");
    }
}
