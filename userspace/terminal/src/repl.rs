use monos_gfx::{
    font::{self, Font},
    text::Origin,
    ui::{widgets, Direction, TextWrap, UIFrame},
    Dimension, Input, Rect,
};
use monoscript::{ast::OwnedValue, ReplContext};
use rooftop::{Window, WindowClient};

use super::{LineType, TerminalInterface};

struct ReplState {
    input: String,
    context: ReplContext,
    interface: TerminalInterface,
    ui: UIFrame,
}

impl ReplState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            context: ReplContext::new(),
            interface: TerminalInterface::new(),
            ui: UIFrame::new(Direction::BottomToTop),
        }
    }
}

pub fn run() -> ! {
    let mut window_client = WindowClient::new("desktop.windows", ReplState::new()).unwrap();

    window_client.create_window("terminal", Dimension::new(320, 240), render);

    loop {
        window_client.update();
        syscall::yield_();
    }
}

fn render(window: &mut Window, state: &mut ReplState, mut input: Input) {
    window.clear();

    let rect = Rect::from_dimensions(window.dimensions()).shrink(2);

    state.ui.draw_frame(window, rect, &mut input, |ui| {
        ui.gap(0);

        let textbox = widgets::Textbox::<font::Glean>::new(&mut state.input)
            .wrap(TextWrap::Enabled { hyphenate: false });
        if ui.add(textbox).submitted {
            state
                .interface
                .add_line(state.input.clone(), LineType::Input);

            match state.context.execute(&state.input, &mut state.interface) {
                Ok(OwnedValue::None) => {}
                Ok(value) => {
                    state
                        .interface
                        .add_line(format!("{:?}", value), LineType::Output);
                }
                Err(err) => {
                    state
                        .interface
                        .add_line(format!("{}", err), LineType::Error);
                }
            }

            state.input.clear();
        }

        ui.add(
            widgets::ScrollableLabel::<font::Glean, _>::new_iter(
                state.interface.lines.iter().map(|line| line.as_str()),
                Origin::Bottom,
            )
            .wrap(TextWrap::Enabled { hyphenate: false })
            .scroll_y(rect.height() - font::Glean::CHAR_HEIGHT - 4)
            .text_colors(state.interface.line_colors.as_slice()),
        );
    });
}
