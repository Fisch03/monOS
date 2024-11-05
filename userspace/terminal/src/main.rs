#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{font, Color, Dimension, Framebuffer, Input, Position, Rect};
use rooftop::WindowClient;

use monos_std::collections::VecDeque;

struct Terminal {
    lines: VecDeque<String>,
    frames: u64,
}

impl Terminal {
    fn new() -> Self {
        Terminal {
            lines: VecDeque::new(),
            frames: 0,
        }
    }
}

#[no_mangle]
fn main() {
    println!("terminal started!");

    let mut window_client = WindowClient::new("desktop.windows", Terminal::new()).unwrap();
    window_client.create_window("terminal1", Dimension::new(100, 100), render1);
    window_client.create_window("terminal2", Dimension::new(100, 100), render2);
    window_client.create_window("terminal3", Dimension::new(100, 100), render3);

    loop {
        window_client.update();
        syscall::yield_();
    }
}

fn render1(app: &mut Terminal, fb: &mut Framebuffer, input: Input) {
    app.frames += 1;

    fb.draw_rect(
        Rect::from_dimensions(fb.dimensions()),
        Color::new(255, 0, 0),
    );

    fb.draw_str::<font::Glean>(
        Color::new(255, 255, 255),
        "good mononing!",
        Position::new(15, (app.frames % 100) as i64),
    );
}

fn render2(app: &mut Terminal, fb: &mut Framebuffer, input: Input) {
    fb.draw_rect(
        Rect::from_dimensions(fb.dimensions()),
        Color::new(0, 255, 0),
    );

    fb.draw_str::<font::Glean>(
        Color::new(255, 255, 255),
        "hello term2!",
        Position::new(15, ((app.frames + 50) % 100) as i64),
    );
}

fn render3(app: &mut Terminal, fb: &mut Framebuffer, input: Input) {
    fb.draw_rect(
        Rect::from_dimensions(fb.dimensions()),
        Color::new((app.frames % 255) as u8, 0, 255),
    );

    fb.draw_str::<font::Glean>(Color::new(255, 255, 255), "=w=", Position::new(40, 45));
}
