#![no_std]
#![no_main]

use bootloader_api::{config, BootInfo, BootloaderConfig};
use core::arch::asm;

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

pub unsafe fn userspace_prog_1() {
    asm!("nop", options(nostack, preserves_flags));
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    monos_kernel::kernel_init(boot_info);
    gfx::framebuffer().update();

    println!("hello world!! :D\nthis is a new line");
    println!();

    interrupts::breakpoint();

    println!();

    process::spawn(userspace_prog_1);

    loop {
        gfx::framebuffer().update();
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

    //always print to serial first. the screen might not be
    //initialized yet
    dbg!(info);

    // eprintln!("oh noes! the kernel {}", info);

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
