#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

mod gdt;
mod gfx;
mod interrupts;
mod mem;
mod serial;
mod utils;

fn kernel_init() {
    serial::init(); // todo: move down

    gdt::init();
    interrupts::init();
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel_init();

    if let Some(raw_fb) = boot_info.framebuffer.as_mut() {
        gfx::init(raw_fb);

        println!("hello world!! :D\nthis is a new line");
        println!();

        interrupts::breakpoint();

        // fn stack_overflow() {
        //     stack_overflow();
        // }
        // stack_overflow();
        println!();

        // unsafe {
        //     *(0xdeadbeef as *mut u8) = 42;
        // };

        // panic!("terrible things");
    }

    loop {
        x86_64::instructions::hlt();
    }
}
bootloader_api::entry_point!(kernel_main);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("oh noes! the kernel {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
