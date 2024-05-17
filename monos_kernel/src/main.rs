#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{config, BootInfo, BootloaderConfig};
use core::arch::asm;
use core::panic::PanicInfo;

mod arch;
mod gdt;
mod gfx;
mod interrupts;
mod mem;
mod serial;
mod utils;

use mem::VirtualAddress;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(config::Mapping::Dynamic);

    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_init(_boot_info: &BootInfo) {
    gdt::init();
    interrupts::init();
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info);

    if let Some(raw_fb) = boot_info.framebuffer.as_mut() {
        gfx::init(raw_fb);

        let phys_mem_offset = boot_info.physical_memory_offset.as_ref().unwrap();
        let phys_mem_offset = VirtualAddress::new(*phys_mem_offset);

        println!("hello world!! :D\nthis is a new line");
        println!();

        let addresses = [
            // the identity-mapped vga buffer page
            0xb8000,
            // some code page
            0x201008,
            // some stack page
            0x0100_0020_1a10,
            // virtual address mapped to physical address 0
            phys_mem_offset.as_u64(),
        ];

        for &address in &addresses {
            let virt = VirtualAddress::new(address);
            let phys = unsafe { mem::paging::translate_addr(virt, phys_mem_offset) };
            println!("{:?} -> {:?}", virt, phys);
        }

        interrupts::breakpoint();

        println!();

        // unsafe {
        //     *(0xdeadbeef as *mut u8) = 42;
        // };
    }

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //always print to serial first. the screen might not be
    //initialized yet
    dbg!(info);

    eprintln!("oh noes! the kernel {}", info);

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
