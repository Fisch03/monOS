#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(prelude_import)]

// import the custom standard library everywhere in the project
#[prelude_import]
#[allow(unused_imports)]
use monos_std::prelude::*;

use monos_gfx::{
    font::{self, Font},
    text::Origin,
    ui::{widgets, Direction, TextWrap, UIFrame},
    Color, Dimension, Framebuffer, Input, Rect,
};
use monoscript::{ast, ReplContext, ReplInterface};
use rooftop::WindowClient;

use monos_std::collections::VecDeque;

struct Terminal {
    interface: TerminalInterface,
    input: String,
    ui: UIFrame,
    context: ReplContext,
}

enum LineType {
    Input,
    Output,
    Error,
}
impl LineType {
    fn color(&self) -> Color {
        match self {
            LineType::Input => Color::new(255, 255, 255),
            LineType::Output => Color::new(150, 150, 150),
            LineType::Error => Color::new(255, 0, 0),
        }
    }
}

struct TerminalInterface {
    lines: VecDeque<String>,
    line_colors: VecDeque<Color>,
}
impl TerminalInterface {
    fn add_line(&mut self, line: String, line_type: LineType) {
        let line = match line_type {
            LineType::Input => format!("> {}", line),
            LineType::Error => format!("! {}", line),
            LineType::Output => line,
        };
        self.lines.push_back(line);
        self.line_colors.push_back(line_type.color());
    }
}
impl ReplInterface for TerminalInterface {
    fn print(&mut self, message: &str) {
        self.add_line(message.to_string(), LineType::Output);
    }
}

impl Terminal {
    fn new() -> Self {
        Terminal {
            ui: UIFrame::new(Direction::BottomToTop),
            input: String::new(),
            context: ReplContext::new(),
            interface: TerminalInterface {
                lines: VecDeque::new(),
                line_colors: VecDeque::new(),
            },
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

fn render(app: &mut Terminal, fb: &mut Framebuffer, mut input: Input) {
    fb.clear();

    let rect = Rect::from_dimensions(fb.dimensions()).shrink(2);

    app.ui.draw_frame(fb, rect, &mut input, |ui| {
        ui.gap(0);

        let textbox = widgets::Textbox::<font::Glean>::new(&mut app.input)
            .wrap(TextWrap::Enabled { hyphenate: false });
        if ui.add(textbox).submitted {
            app.interface.add_line(app.input.clone(), LineType::Input);

            match app.context.execute(&app.input, &mut app.interface) {
                Ok(ast::OwnedValue::None) => {}
                Ok(value) => {
                    app.interface
                        .add_line(format!("{:?}", value), LineType::Output);
                }
                Err(err) => {
                    app.interface.add_line(format!("{}", err), LineType::Error);
                }
            }

            app.input.clear();
        }

        ui.add(
            widgets::ScrollableLabel::<font::Glean, _>::new_iter(
                app.interface.lines.iter().map(|line| line.as_str()),
                Origin::Bottom,
            )
            .wrap(TextWrap::Enabled { hyphenate: false })
            .scroll_y(rect.height() - font::Glean::CHAR_HEIGHT - 4)
            .text_colors(app.interface.line_colors.make_contiguous()),
        );
    });
}
