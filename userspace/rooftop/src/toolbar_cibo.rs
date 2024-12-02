mod speech_widget;
use speech_widget::SpeechWidget;

use monos_gfx::{
    types::*,
    ui::{Direction, MarginMode, UIFrame},
    Framebuffer, Input, Rect,
};
use monos_std::collections::VecDeque;

pub const MESSAGE_LINGER_TIME: u64 = 10000;

pub struct ToolbarCibo {
    ui: UIFrame,
    messages: VecDeque<(String, u64)>,
}

impl ToolbarCibo {
    pub fn new() -> Self {
        Self {
            ui: UIFrame::new(Direction::BottomToTop),
            messages: VecDeque::new(),
        }
    }

    pub fn rect(&self, fb: &Framebuffer) -> Rect {
        Rect::new(
            Position::new(fb.dimensions().width as i64 - 118, 0),
            Position::new(
                fb.dimensions().width as i64 - 10,
                fb.dimensions().height as i64 - 40,
            ),
        )
    }

    pub fn draw(&mut self, fb: &mut Framebuffer, clear_fb: &Framebuffer) {
        if self.messages.is_empty() {
            return;
        }

        let time = syscall::get_time();
        while let Some(message) = self.messages.front() {
            if time - message.1 > MESSAGE_LINGER_TIME {
                self.messages.pop_front();
                fb.clear_region(&self.rect(&fb).grow(1), &clear_fb);
            } else {
                break;
            }
        }

        fb.clear_region(&self.rect(&fb).grow(2), &clear_fb);
        self.ui
            .draw_frame(fb, self.rect(fb), &mut Input::default(), |ui| {
                ui.margin(MarginMode::Grow);
                for message in self.messages.iter().rev() {
                    ui.add(SpeechWidget::new(&message.0));
                }
            });
    }

    pub fn add_message(&mut self, message: &str) {
        self.messages
            .push_back((String::from(message), syscall::get_time()));
    }
}
