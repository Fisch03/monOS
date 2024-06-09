#![no_std]
#![no_main]
#![feature(naked_functions)]

use bootloader_api::{config, BootInfo, BootloaderConfig};
use core::arch::asm;

use fs::*;
use monos_kernel::*;

extern crate alloc;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(config::Mapping::Dynamic);
    config.mappings.dynamic_range_start = Some(0xffff_8000_0000_0000);

    // currently the frame allocator bitmap lives fully on the stack, so we need a bigger stack
    config.kernel_stack_size = 1024 * 1024;

    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    monos_kernel::kernel_init(boot_info);

    println!("hello world!! :D\nthis is a new line");
    println!();

    interrupts::breakpoint();

    println!();

    let hello_world = {
        let mut fs = fs().lock();
        let hello_world = fs
            .iter_root_dir()
            .get_entry("bin/hello_world")
            .unwrap()
            .as_file()
            .unwrap();

        let mut data = alloc::vec![0u8; hello_world.size()];
        hello_world.read_all(data.as_mut_slice());
        data
    };

    process::spawn(&hello_world.as_slice());

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))] // avoid stupid duplicate lang item error
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupts::disable();

    dbg!(info);

    if let Some(mut fb_guard) = framebuffer::get() {
        let fb = unsafe { fb_guard.now_or_never() };

        use core::fmt::Write;
        write!(fb, "oh noes! the kernel {}", info).unwrap();
        fb.update();
    }

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
