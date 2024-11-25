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

use monos_gfx::{
    font::{self, Font},
    text::Origin,
    ui::{widgets, Direction, TextWrap, UIFrame},
    Color, Dimension, Framebuffer, Input, Rect,
};
use rooftop::WindowClient;

extern "C" {
    fn doomgeneric_Create(argc: i32, argv: *const *const u8);
    fn doomgeneric_Tick();
}

#[no_mangle]
pub unsafe extern "C" fn DG_Init() {}
#[no_mangle]
pub unsafe extern "C" fn DG_DrawFrame() {
    println!("DG_DrawFrame");
}
#[no_mangle]
pub unsafe extern "C" fn DG_SleepMs(ms: u32) {
    println!("DG_SleepMs");
}
#[no_mangle]
pub unsafe extern "C" fn DG_GetTicksMs() -> u32 {
    todo!("DG_GetTicksMs");
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
    let mut window_client = WindowClient::new("desktop.windows", ()).unwrap();
    window_client.create_window("terminal", Dimension::new(640, 400), render);

    let args: [&core::ffi::CStr; 4] = [c"bin/doom", c"-iwad", c"data/doom1.wad", c"-nosound"];
    let args = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    unsafe { doomgeneric_Create(args.len() as i32, args.as_ptr() as *const *const u8) };

    loop {
        window_client.update();
        unsafe { doomgeneric_Tick() };
    }
}

fn render(app: &mut (), fb: &mut Framebuffer, mut input: Input) {}
