#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_std::dev::mouse::MouseState;

use monos_gfx::{
    framebuffer::{FramebufferRequest, FramebufferResponse},
    input::Input,
    ui::*,
    Framebuffer, Position, Rect,
};

#[no_mangle]
fn main() {
    let fb_channel = syscall::connect("sys.framebuffer").unwrap();
    let mut fb: Option<Framebuffer> = None;
    // TODO: send_sync
    syscall::send(fb_channel, FramebufferRequest::Open(&mut fb));
    unsafe { syscall::receive_as::<FramebufferResponse>(fb_channel).unwrap() };

    let mut fb = fb.unwrap();
    //TODO: uhhhhhhhh for some reason removing this print breaks the framebuffer dimensions. i
    //really should look into that
    println!(
        "initializing desktop environment with a resolution of {}x{}",
        fb.scaled_dimensions().width,
        fb.scaled_dimensions().height
    );

    let fb_rect = Rect::from_dimensions(fb.scaled_dimensions());
    let mut ui_rect = Rect::from_dimensions(fb.scaled_dimensions());

    let mouse_channel = syscall::connect("sys.mouse").unwrap();
    let keyboard_channel = syscall::connect("sys.keyboard").unwrap();

    let mut ui_frame = UIFrame::new(Direction::TopToBottom);
    let mut input = Input::default();

    loop {
        while let Some(msg) = syscall::receive_any() {
            if msg.sender == mouse_channel {
                input
                    .mouse
                    .update(unsafe { MouseState::from_message(&msg).unwrap() }, fb_rect);

                ui_rect.max = input.mouse.position;
            } else if msg.sender == keyboard_channel {
                let key = msg.data.0 as u8 as char;
                println!("Key: {:?}", key);
            }
        }

        fb.clear();

        ui_frame.draw_frame(&mut fb, ui_rect, &input, |ui| {
            ui.label("good mononing!!!\n");

            if ui.button("click me").clicked {
                println!("button clicked!");
            }

            ui.button("margin minimum, padding gap 2");
            ui.padding(PaddingMode::Gap(2));
            ui.margin(MarginMode::Grow);
            ui.button("margin grow, padding gap 2");
            ui.padding(PaddingMode::Fill);
            ui.button("margin grow, padding fill");
            ui.padding(PaddingMode::Gap(20));
            ui.button("margin grow, padding gap 20");
        });
        draw_cursor(&mut fb, input.mouse.position);

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
