#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]
#![feature(c_variadic)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

extern crate alloc;

mod libc;

extern "C" {
    fn doomgeneric_Create(argc: i32, argv: *const *const u8);
    fn doomgeneric_Tick();
}

#[no_mangle]
pub unsafe extern "C" fn DG_Init() {}
#[no_mangle]
pub unsafe extern "C" fn DG_DrawFrame() {
    todo!("DG_DrawFrame");
}
#[no_mangle]
pub unsafe extern "C" fn DG_SleepMs(ms: u32) {
    todo!("DG_SleepMs");
}
#[no_mangle]
pub unsafe extern "C" fn DG_GetTicksMs() -> u32 {
    todo!("DG_GetTicksMs");
}
#[no_mangle]
pub unsafe extern "C" fn DG_GetKey() -> i32 {
    todo!("DG_GetKey");
}
#[no_mangle]
pub unsafe extern "C" fn DG_SetWindowTitle(title: *const i8) {
    let title = core::ffi::CStr::from_ptr(title);
    println!("window title: {}", title.to_str().unwrap());
}

#[no_mangle]
fn main() {
    let args: [&core::ffi::CStr; 3] = [c"bin/doom", c"-iwad", c"data/doom1.wad"];
    let args = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    unsafe { doomgeneric_Create(args.len() as i32, args.as_ptr() as *const *const u8) };

    loop {
        unsafe { doomgeneric_Tick() };
    }
}
