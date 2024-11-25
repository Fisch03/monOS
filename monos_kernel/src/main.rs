#![no_std]
#![no_main]
#![feature(naked_functions)]

use bootloader_api::BootInfo;
use core::arch::asm;

use monos_kernel::*;

extern crate alloc;

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(raw_fb) = boot_info.framebuffer.as_mut() {
        draw_boot_logo(raw_fb);
    }

    monos_kernel::kernel_init(boot_info);

    // start the desktop environment
    interrupts::without_interrupts(|| {
        process::spawn("bin/rooftop").expect("failed to start desktop environment");
    });

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

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("\n\n");
    println!("oh noes, the kernel panicked!");
    kernel_panic(info)
}
