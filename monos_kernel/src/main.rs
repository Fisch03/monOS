#![no_std]
#![no_main]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

mod gfx;

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(raw_fb) = boot_info.framebuffer.as_mut() {
        gfx::init(raw_fb);
        println!("hello world!! :D\nthis is a new line");
        println!();

        panic!("terrible things");
    }

    loop {}
}
bootloader_api::entry_point!(kernel_main);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("oh noes! the kernel {}", info);
    loop {}
}
