use super::Lines;
use crate::input::*;
use crate::types::*;
use crate::ui::*;
use crate::Font;
use core::marker::PhantomData;

pub struct Textbox<'a, F>
where
    F: Font,
{
    id: u64,
    text: &'a mut String,
    wrap: TextWrap,
    state: TextboxState,
    font: PhantomData<F>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TextboxState {
    cursor: usize,
    selection: Option<usize>,
}

impl<'a, F: Font> Textbox<'a, F> {
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

        Self {
            id,
            text,
            state,
            wrap: TextWrap::Disabled,
            font: PhantomData,
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }
}

impl<F: Font> UIElement for Textbox<'_, F> {
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

        let max_dimensions =
            Dimension::new(context.placer.max_width(), context.fb.dimensions().height);

        let lines = Lines::<F>::layout(self.text, self.wrap, max_dimensions);

        let line_dimensions = lines.dimensions();

        let mut result = context.alloc_space(line_dimensions);
        result.submitted = submitted;

        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        lines.draw(context.fb, lines_rect.min, Color::new(255, 255, 255));

        let cursor_pos = lines_rect.min + lines.char_position(self.state.cursor);
        let cursor_rect = Rect::new(
            cursor_pos,
            Position::new(cursor_pos.x + 1, cursor_pos.y + F::CHAR_HEIGHT as i64),
        );
        context
            .fb
            .draw_rect(&cursor_rect, &Color::new(255, 255, 255));

        context.state_insert(self.id, self.state);

        result
    }
}
