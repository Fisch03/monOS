#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{Color, Dimension, Framebuffer, Input};
use rooftop::WindowClient;

use monos_std::collections::VecDeque;

struct Terminal {
    lines: VecDeque<String>,
}

impl Terminal {
    fn new() -> Self {
        Terminal {
            lines: VecDeque::new(),
        }
    }
}

#[no_mangle]
fn main() {
    println!("terminal started!");

    let mut window_client = WindowClient::new("desktop.windows", Terminal::new()).unwrap();
    window_client.create_window("terminal", Dimension::new(320, 240), render);

    loop {
        window_client.update();
        syscall::yield_();
    }
}

fn render(app: &mut Terminal, fb: &mut Framebuffer, input: Input) {
    fb.clear();

    fb.draw_pixel(input.mouse.position, Color::new(255, 255, 255));
}
