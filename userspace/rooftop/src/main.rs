#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::Framebuffer;

#[no_mangle]
fn main() {
    let mut fb = syscall::open_fb().unwrap();

    loop {
        fb.clear();

        draw_cursor(&mut fb);

        syscall::submit_frame(&fb);
    }
}

fn draw_cursor(fb: &mut Framebuffer) {}
