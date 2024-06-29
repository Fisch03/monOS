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
    //unsafe { syscall::receive_as::<FramebufferResponse>(fb_channel).unwrap() };

    let mut fb = fb.unwrap();
    //TODO: uhhhhhhhh for some reason removing this print breaks the framebuffer dimensions.
    //i really should look into that
    println!(
        "initializing desktop environment with a resolution of {}x{}",
        fb.scaled_dimensions().width,
        fb.scaled_dimensions().height
    );
    let fb_rect = Rect::from_dimensions(fb.scaled_dimensions());

    // println!("Framebuffer len: {:0x}", fb.buffer().len());
    // let mut clear_fb_buffer = vec![0; fb.buffer().len()];
    // let clear_fb = create_clear_fb(&fb, &mut clear_fb_buffer);

    let mouse_channel = syscall::connect("sys.mouse").unwrap();
    let keyboard_channel = syscall::connect("sys.keyboard").unwrap();
    let mut input = Input::default();

    let taskbar_ui_rect = Rect::new(
        Position::new(0, fb.scaled_dimensions().height as i64 - 20),
        Position::new(
            fb.scaled_dimensions().width as i64,
            fb.scaled_dimensions().height as i64,
        ),
    );
    let mut taskbar_ui = UIFrame::new(Direction::LeftToRight);

    let test_icon = monos_gfx::Image::from_ppm(include_bytes!("../assets/test_icon.ppm"))
        .expect("failed to load image");
    //println!("test_icon: {:?}", test_icon);

    loop {
        input.mouse.update();
        while let Some(msg) = syscall::receive_any() {
            if msg.sender == mouse_channel {
                if let Some(mouse_state) = unsafe { MouseState::from_message(&msg) } {
                    input.mouse.update_new(mouse_state, fb_rect);
                }
            } else if msg.sender == keyboard_channel {
                let key = msg.data.0 as u8 as char;
                println!("Key: {:?}", key);
            }
        }

        // fb.clear_with(&clear_fb);
        fb.clear();

        taskbar_ui.draw_frame(&mut fb, taskbar_ui_rect, &input, |ui| {
            ui.margin(MarginMode::Grow);

            ui.img_button(&test_icon);
        });
        draw_cursor(&mut fb, input.mouse.position);

        syscall::send(fb_channel, FramebufferRequest::SubmitFrame(&fb));
    }
}

fn create_clear_fb<'a>(main_fb: &Framebuffer, buffer: &'a mut Vec<u8>) -> Framebuffer<'a> {
    let mut clear_fb = Framebuffer::new(
        buffer,
        main_fb.actual_dimensions(),
        main_fb.format().clone(),
    );

    let taskbar = monos_gfx::Image::from_ppm(include_bytes!("../assets/taskbar.ppm"))
        .expect("failed to load image");
    println!("taskbar: {:?}", taskbar);

    clear_fb.draw_img(
        &taskbar,
        &Position::new(
            0,
            (clear_fb.scaled_dimensions().height - taskbar.dimensions().height) as i64,
        ),
    );

    clear_fb
}

fn draw_cursor(fb: &mut Framebuffer, pos: Position) {
    fb.draw_char::<monos_gfx::fonts::Cozette>(
        &monos_gfx::Color::new(255, 255, 255),
        '\u{F55A}',
        &pos,
    );
}
