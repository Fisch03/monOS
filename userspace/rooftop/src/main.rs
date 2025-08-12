#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

mod windowing;
use windowing::server::WindowServer;

mod toolbar_cibo;
use toolbar_cibo::ToolbarCibo;

mod desktop;
use desktop::Desktop;

use monos_std::dev::{keyboard::KeyEvent, mouse::MouseState};

use monos_gfx::{
    framebuffer::{FramebufferRequest, FramebufferResponse},
    input::Input,
    text::font::Cozette,
    Framebuffer, Position, Rect,
};

const WELCOME_MESSAGES: [&str; 3] = [
    "welcome to monOS!",
    "if youre new, check out the welcome file on the desktop!!",
    "have fun exploring!",
];

#[no_mangle]
fn main() {
    let fb_channel = syscall::connect("sys.framebuffer").unwrap();
    let mut fb: Option<Framebuffer> = None;

    // TODO: send_sync
    syscall::send(fb_channel, FramebufferRequest::Open(&mut fb));
    unsafe { syscall::receive_as::<FramebufferResponse>(fb_channel).unwrap() };

    let mut fb = fb.unwrap();

    let mouse_rect = Rect::new(
        Position::new(0, 0),
        Position::new(
            fb.dimensions().width as i64 - 6,
            fb.dimensions().height as i64 - 9,
        ),
    );

    let mouse_channel = syscall::connect("sys.mouse").unwrap();
    let keyboard_channel = syscall::connect("sys.keyboard").unwrap();

    let mut input = Input::default();

    //TODO: refresh desktop entries every once in a while
    let mut clear_fb_buffer = Vec::new();
    let mut paint_fb_buffer = Vec::new();
    let mut desktop = Desktop::new(&fb, &mut clear_fb_buffer, &mut paint_fb_buffer);

    let window_list_rect = Rect::new(
        Position::new(2, fb.dimensions().height as i64 - 22),
        Position::new(
            fb.dimensions().width as i64 - 2,
            fb.dimensions().height as i64,
        ),
    );

    let mut window_server = WindowServer::new("desktop.windows");

    let mut toolbar_cibo = ToolbarCibo::new();
    let mut next_message = syscall::get_time() + 2500;
    let mut curr_message: i64 = -1;

    let mut old_mouse_pos = Position::new(0, 0);

    fb.clear_with(&desktop);
    println!("starting event loop");

    // desktop
    //     .paint()
    //     .splat(Position::new(100, 100), monos_gfx::Color::new(255, 0, 0));
    // desktop
    //     .paint()
    //     .splat(Position::new(100, 150), monos_gfx::Color::new(255, 0, 0));
    // desktop
    //     .paint()
    //     .splat(Position::new(100, 200), monos_gfx::Color::new(255, 0, 0));
    // desktop
    //     .paint()
    //     .splat(Position::new(100, 250), monos_gfx::Color::new(255, 0, 0));

    //syscall::spawn("bin/terminal");

    loop {
        while let Some(msg) = syscall::receive_any() {
            if msg.sender == mouse_channel {
                if let Some(mouse_state) = unsafe { MouseState::from_message(msg) } {
                    input.mouse.update_new(mouse_state, mouse_rect);
                }
            } else if msg.sender == keyboard_channel {
                if let Some(key_event) = unsafe { KeyEvent::from_message(msg) } {
                    input.keyboard.keys.push(key_event);
                }
            } else {
                // safety: since the only other channel is the window server we know this is a window message
                unsafe { window_server.handle_message(msg) };
            }
        }

        let old_mouse_rect = Rect::new(old_mouse_pos, old_mouse_pos + Position::new(6, 9));
        if input.mouse.moved() {
            fb.clear_region(&old_mouse_rect, &desktop);
            old_mouse_pos = input.mouse.position;
        }

        let needs_clear = desktop.update(&mut input);
        if needs_clear {
            fb.clear_with(&desktop);
        }

        let time = syscall::get_time();
        if time < WELCOME_MESSAGES.len() as u64 * 2500 + toolbar_cibo::MESSAGE_LINGER_TIME + 3000 {
            if time > next_message {
                curr_message += 1;
                if let Some(message) = WELCOME_MESSAGES.get(curr_message as usize) {
                    toolbar_cibo.add_message(message);
                }
                next_message = syscall::get_time() + 2500;
            }
        }
        toolbar_cibo.draw(&mut fb, &desktop);

        window_server.draw_window_list(&mut fb, window_list_rect, &mut input, &desktop);
        let res = window_server.draw(&mut fb, &mut input, &desktop, old_mouse_rect);

        if !res.hide_cursor {
            draw_cursor(&mut fb, input.mouse.position);
        }

        syscall::send(fb_channel, FramebufferRequest::SubmitFrame(&fb));
        input.clear();

        syscall::yield_();
    }
}

fn draw_cursor(fb: &mut Framebuffer, mut pos: Position) {
    pos.y -= 4;
    pos.x -= 1;

    fb.draw_char::<Cozette>(monos_gfx::Color::new(255, 255, 255), '\u{F55A}', &pos);
}
