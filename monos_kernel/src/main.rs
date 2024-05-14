#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

mod gfx;
mod interrupts;
mod utils;

fn kernel_init() {
    interrupts::init_idt();
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

        unsafe {
            *(0xdeadbeef as *mut u8) = 42;
        };

        // panic!("terrible things");
    }

    loop {}
}
bootloader_api::entry_point!(kernel_main);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("oh noes! the kernel {}", info);
    loop {}
}
