#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{
    framebuffer::{FramebufferRequest, FramebufferResponse},
    ui::*,
    Framebuffer, Position, Rect,
};

#[no_mangle]
fn main() {
    let fb_channel = syscall::connect("sys.framebuffer").unwrap();
    let mut fb: Option<Framebuffer> = None;
    // TODO: send_sync
    syscall::send(fb_channel, FramebufferRequest::Open(&mut fb));
    let mut fb = fb.unwrap();

    let fb_rect = Rect::from_dimensions(fb.scaled_dimensions());

    let mouse_channel = syscall::connect("sys.mouse").unwrap();
    println!("Mouse channel: {:?}", mouse_channel);
    let keyboard_channel = syscall::connect("sys.keyboard").unwrap();
    println!("Keyboard channel: {:?}", mouse_channel);

    let mut cursor_pos = Position::new(10, 10);

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

                // fb_rect.max = cursor_pos;
            } else if msg.sender == keyboard_channel {
                let key = msg.data.0 as u8 as char;
                println!("Key: {:?}", key);
            }
        }

        fb.clear();

        ui_frame.draw_frame(&mut fb, fb_rect, |ui| {
            ui.label("good mononing!!!\n");

            // if ui.button("click me").clicked {
            //     println!("button clicked!");
            // }

            ui.button("margin minimum, padding gap 0");
            ui.margin(MarginMode::AtLeast(250));
            ui.button("margin least 250, padding gap 0");
            ui.margin(MarginMode::Grow);
            ui.button("margin grow, padding gap 0");
            ui.padding(PaddingMode::Fill);
            ui.button("margin grow, padding fill");
            ui.padding(PaddingMode::Gap(20));
            ui.button("margin grow, padding gap 20");
        });
        draw_cursor(&mut fb, cursor_pos);

        syscall::send(fb_channel, FramebufferRequest::SubmitFrame(&fb));
    }
}

fn draw_cursor(fb: &mut Framebuffer, pos: Position) {
    fb.draw_char::<monos_gfx::fonts::Cozette>(
        &monos_gfx::Color::new(255, 255, 255),
        '\u{F55A}',
        &pos,
    );
}
