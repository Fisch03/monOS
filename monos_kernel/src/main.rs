#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;
use core::arch::asm;
use core::panic::PanicInfo;

mod arch;
mod gdt;
mod gfx;
mod interrupts;
mod mem;
mod serial;
mod utils;

fn kernel_init(boot_info: &BootInfo) {
    if let Some(physical_memory_offset) = boot_info.physical_memory_offset.as_ref() {
        mem::set_physical_memory_offset(*physical_memory_offset);
    }

    gdt::init();
    interrupts::init();
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init(boot_info);

    if let Some(raw_fb) = boot_info.framebuffer.as_mut() {
        gfx::init(raw_fb);

        println!("hello world!! :D\nthis is a new line");
        println!();

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
bootloader_api::entry_point!(kernel_main);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("oh noes! the kernel {}", info);

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
