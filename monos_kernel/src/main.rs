#![no_std]
#![no_main]

use bootloader_api::{config, BootInfo, BootloaderConfig};
use core::arch::asm;
use core::panic::PanicInfo;

use monos_kernel::*;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(config::Mapping::Dynamic);

    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    monos_kernel::kernel_init(boot_info);

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
