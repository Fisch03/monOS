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
    if let Some(raw_fb) = boot_info.framebuffer.as_mut() {
        draw_boot_logo(raw_fb);
    }

    monos_kernel::kernel_init(boot_info);

    // start the desktop environment
    // let rooftop = {
    //     let mut fs = fs().lock();
    //     let rooftop = fs
    //         .iter_root_dir()
    //         .get_entry("bin/rooftop")
    //         .unwrap()
    //         .as_file()
    //         .unwrap();
    //
    //     let mut data = alloc::vec![0u8; rooftop.size()];
    //     rooftop.read_all(data.as_mut_slice());
    //     data
    // };

    // process::spawn(&rooftop.as_slice());

    let hello = {
        let mut fs = fs().lock();
        let file = fs
            .iter_root_dir()
            .get_entry("bin/hello_world")
            .unwrap()
            .as_file()
            .unwrap();

        let mut data = alloc::vec![0u8; file.size()];
        file.read_all(data.as_mut_slice());
        data
    };

    let bye = {
        let mut fs = fs().lock();
        let file = fs
            .iter_root_dir()
            .get_entry("bin/bye_world")
            .unwrap()
            .as_file()
            .unwrap();

        let mut data = alloc::vec![0u8; file.size()];
        file.read_all(data.as_mut_slice());
        data
    };

    process::spawn(&hello.as_slice());
    process::spawn(&bye.as_slice());

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

fn draw_boot_logo(raw_fb: &mut bootloader_api::info::FrameBuffer) {
    const LOGO_WIDTH: usize = 270;
    const LOGO_HEIGHT: usize = 75;

    let logo = include_bytes!("../assets/boot_logo.ppm");

    let info = raw_fb.info();
    let raw_fb = raw_fb.buffer_mut();
    raw_fb.fill(0);

    let y_start = (info.height - LOGO_HEIGHT) / 2;
    let x_start = (info.width - LOGO_WIDTH) / 2;

    for y in 0..LOGO_HEIGHT {
        for x in 0..LOGO_WIDTH {
            let offset = (y * LOGO_WIDTH + x) * 3;
            let r = logo[offset] as u8;
            let g = logo[offset + 1] as u8;
            let b = logo[offset + 2] as u8;

            let buffer_index = ((y + y_start) * info.stride + (x + x_start)) * info.bytes_per_pixel;
            raw_fb[buffer_index] = r;
            raw_fb[buffer_index + 1] = g;
            raw_fb[buffer_index + 2] = b;
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

        // TODO

        fb.update();
    }

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
