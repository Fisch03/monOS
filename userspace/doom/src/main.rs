#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]
#![feature(c_variadic)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

mod libc;

use monos_gfx::{Dimension, Framebuffer, FramebufferFormat, Input};
use rooftop::WindowClient;

use core::sync::atomic::{AtomicBool, Ordering};

static FRAME_READY: AtomicBool = AtomicBool::new(false);

extern "C" {
    static DG_ScreenBuffer: *mut u8;
    fn doomgeneric_Create(argc: i32, argv: *const *const u8);
    fn doomgeneric_Tick();
}

#[no_mangle]
pub unsafe extern "C" fn DG_Init() {}
#[no_mangle]
pub unsafe extern "C" fn DG_DrawFrame() {
    FRAME_READY.store(true, Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn DG_SleepMs(ms: u32) {
    let target = syscall::get_time() + ms as u64;
    loop {
        if syscall::get_time() >= target {
            break;
        }
        syscall::yield_();
    }
}

#[no_mangle]
pub unsafe extern "C" fn DG_GetTicksMs() -> u32 {
    syscall::get_time() as u32
}
#[no_mangle]
pub unsafe extern "C" fn DG_GetKey() -> i32 {
    // todo!("DG_GetKey");
    0
}
#[no_mangle]
pub unsafe extern "C" fn DG_SetWindowTitle(title: *const i8) {
    let title = core::ffi::CStr::from_ptr(title);
    println!("window title: {}", title.to_str().unwrap());
}

#[no_mangle]
fn main() {
    let args: [&core::ffi::CStr; 4] = [c"bin/doom", c"-iwad", c"data/doom1.wad", c"-nosound"];
    let args = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    unsafe { doomgeneric_Create(args.len() as i32, args.as_ptr() as *const *const u8) };

    let mut window_client = WindowClient::new("desktop.windows", ()).unwrap();
    window_client.create_window("terminal", Dimension::new(320, 200), render);

    loop {
        window_client.update();
    }
}

fn render(_app: &mut (), fb: &mut Framebuffer, _input: Input) {
    while !FRAME_READY.load(Ordering::Relaxed) {
        unsafe { doomgeneric_Tick() };
    }
    println!("frame ready");

    FRAME_READY.store(false, Ordering::Relaxed);

    let doom_fb = unsafe { core::slice::from_raw_parts_mut(DG_ScreenBuffer, 320 * 200 * 3) };
    let doom_fb = Framebuffer::new(
        doom_fb,
        Dimension::new(320, 200),
        FramebufferFormat {
            bytes_per_pixel: 3,
            stride: 320,
            r_position: 0,
            g_position: 1,
            b_position: 2,
            a_position: None,
        },
    );

    fb.clear_with(&doom_fb);
}
