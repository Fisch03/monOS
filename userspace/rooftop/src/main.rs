#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{Framebuffer, Position};

#[no_mangle]
fn main() {
    let mut fb = syscall::open_fb().unwrap();

    let mouse_channel = syscall::connect("sys.mouse").unwrap();
    // println!("Mouse channel: {:?}", mouse_channel);
    // let keyboard_channel = syscall::connect("sys.keyboard").unwrap();
    // println!("Keyboard channel: {:?}", mouse_channel);

    let mut cursor_pos = Position::new(10, 10);

    fb.clear();
    loop {
        while let Some(msg) = syscall::receive(mouse_channel) {
            let x: u32 = msg.data.0 as u32;
            let x = unsafe { core::mem::transmute::<u32, i32>(x) };
            cursor_pos.x += x as i64;

            let y: u32 = msg.data.1 as u32;
            let y = unsafe { core::mem::transmute::<u32, i32>(y) };
            cursor_pos.y += y as i64;
        }

        draw_cursor(&mut fb, cursor_pos);
        syscall::submit_frame(&fb);
    }
}

fn draw_cursor(fb: &mut Framebuffer, pos: Position) {
    for y in 0..16 {
        for x in 0..16 {
            fb.draw_pixel(
                &Position::new(pos.x + x as i64, pos.y + y as i64),
                &monos_gfx::Color::new(255, 255, 255),
            );
        }
    }
}
