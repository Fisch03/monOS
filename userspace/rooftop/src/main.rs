#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{ui::*, Framebuffer, Position, Rect};

#[no_mangle]
fn main() {
    let mut fb = syscall::open_fb().unwrap();
    let mut fb_rect = Rect::from_dimensions(fb.scaled_dimensions());

    let mouse_channel = syscall::connect("sys.mouse").unwrap();
    println!("Mouse channel: {:?}", mouse_channel);
    let keyboard_channel = syscall::connect("sys.keyboard").unwrap();
    println!("Keyboard channel: {:?}", mouse_channel);

    let mut cursor_pos = Position::new(fb_rect.max.x - 10, 10);

    let mut ui_frame = UIFrame::new(Direction::TopToBottom);

    fb.clear();
    loop {
        while let Some(msg) = syscall::receive_any() {
            if msg.sender == mouse_channel {
                let x: u32 = msg.data.0 as u32;
                let x = unsafe { core::mem::transmute::<u32, i32>(x) };
                cursor_pos.x += x as i64;
                cursor_pos.x = cursor_pos.x.max(0).min(fb.scaled_dimensions().width as i64);

                let y: u32 = msg.data.1 as u32;
                let y = unsafe { core::mem::transmute::<u32, i32>(y) };
                cursor_pos.y -= y as i64;
                cursor_pos.y = cursor_pos
                    .y
                    .max(0)
                    .min(fb.scaled_dimensions().height as i64);

                fb_rect.max = cursor_pos;
            } else if msg.sender == keyboard_channel {
                let key = msg.data.0 as u8 as char;
                println!("Key: {:?}", key);
            }
        }

        fb.clear();

        ui_frame.draw_frame(&mut fb, fb_rect, |ui| {
            ui.label("good mononing!!!\n");
            ui.label(
                r#"big text: 
big-long-hyphenated-word

Scawy big no functionality at all UwU 

Senpai,

your little kawaii pwintf impwementation is getting out of contwol.
Pwease have a look at this:

The pwintf function:

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        // TODO: figure out why format!() doesn't work
        let mut s = $crate::prelude::String::new();
        let _ = write!(s, $($arg)*);
        $crate::syscall::print(&s);

    }};
}
"#,
            );
        });
        draw_cursor(&mut fb, cursor_pos);

        syscall::submit_frame(&fb);
    }
}

fn draw_cursor(fb: &mut Framebuffer, pos: Position) {
    fb.draw_char::<monos_gfx::fonts::Cozette>(
        &monos_gfx::Color::new(255, 255, 255),
        '\u{F55A}',
        &pos,
    );
}
