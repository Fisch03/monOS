use super::Lines;
use crate::fonts::{Cozette, Font};
use crate::input::*;
use crate::types::*;
use crate::ui::*;

pub struct Textbox<'a> {
    id: u64,
    text: &'a mut String,
    state: TextboxState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TextboxState {
    cursor: usize,
    selection: Option<usize>,
}

impl<'a> Textbox<'a> {
    pub fn new(text: &'a mut String, context: &mut UIContext) -> Self {
        let id = context.next_id();
        let state = if let Some(state) = context.state_get(id) {
            state
        } else {
            TextboxState {
                cursor: 0,
                selection: None,
            }
        };

        Self { id, text, state }
    }
}

impl UIElement for Textbox<'_> {
    fn draw(mut self, context: &mut UIContext) -> UIResult {
        let mut submitted = false;

        self.state.cursor = self.state.cursor.min(self.text.len());

        //TODO: check for focus
        while let Some(event) = context.input.keyboard.pop_front() {
            match event.state {
                KeyState::Up => continue,
                _ => (),
            }

            match event.key {
                Key::Unicode(c) => {
                    self.text.insert(self.state.cursor, c);
                    self.state.cursor += 1;
                }

                Key::RawKey(RawKey::ArrowLeft) => {
                    if self.state.cursor > 0 {
                        self.state.cursor -= 1;
                    }
                }
                Key::RawKey(RawKey::ArrowRight) => {
                    if self.state.cursor < self.text.len() {
                        self.state.cursor += 1;
                    }
                }

                Key::RawKey(RawKey::Return) => {
                    submitted = true;
                }
                Key::RawKey(RawKey::Backspace) => {
                    if self.state.cursor > 0 {
                        self.text.remove(self.state.cursor - 1);
                        self.state.cursor -= 1;
                    }
                }
                Key::RawKey(RawKey::Delete) => {
                    if self.state.cursor < self.text.len() {
                        self.text.remove(self.state.cursor);
                    }
                }

                _ => (),
            }
        }

        let line = Lines::layout_single_line(&self.text, context.placer.max_width());

        let line_dimensions = line.dimensions();

        let mut result = context.alloc_space(line_dimensions);
        result.submitted = submitted;

        let lines_rect = Rect::centered_in(result.rect, line_dimensions);
        line.draw(context.fb, lines_rect.min, Color::new(255, 255, 255));

        let cursor_x = lines_rect.min.x + (self.state.cursor as i64 * Cozette::CHAR_WIDTH as i64);
        let cursor_rect = Rect::new(
            Position::new(cursor_x, lines_rect.min.y),
            Position::new(cursor_x + 1, lines_rect.max.y),
        );
        context
            .fb
            .draw_rect(&cursor_rect, &Color::new(255, 255, 255));

        context.state_insert(self.id, self.state);

        result
    }
}
