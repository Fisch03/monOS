use crate::input::*;
use crate::text::{Font, Lines};
use crate::types::*;
use crate::ui::*;
use core::marker::PhantomData;

pub struct Textbox<'a, F>
where
    F: Font,
{
    text: &'a mut String,
    wrap: TextWrap,
    char_limit: Option<usize>,
    font: PhantomData<F>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct TextboxState {
    cursor: usize,
    selection: Option<usize>,
}

impl<'a, F: Font> Textbox<'a, F> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            wrap: TextWrap::Disabled,
            font: PhantomData,
            char_limit: None,
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }
}

impl<F: Font> UIElement for Textbox<'_, F> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let id = context.next_id();
        let mut state: TextboxState = context.state_get(id).unwrap_or_default();

        let mut submitted = false;

        state.cursor = state.cursor.min(self.text.len());

        //TODO: check for focus
        while let Some(event) = context.input.keyboard.pop_front() {
            match event.state {
                KeyState::Up => continue,
                _ => (),
            }

            match event.key {
                Key::Unicode(c) => {
                    if let Some(limit) = self.char_limit {
                        if self.text.len() >= limit {
                            continue;
                        }
                    }

                    self.text.insert(state.cursor, c);
                    state.cursor += 1;
                }

                Key::RawKey(RawKey::ArrowLeft) => {
                    if state.cursor > 0 {
                        state.cursor -= 1;
                    }
                }
                Key::RawKey(RawKey::ArrowRight) => {
                    if state.cursor < self.text.len() {
                        state.cursor += 1;
                    }
                }

                Key::RawKey(RawKey::Return) => {
                    submitted = true;
                }
                Key::RawKey(RawKey::Backspace) => {
                    if state.cursor > 0 {
                        self.text.remove(state.cursor - 1);
                        state.cursor -= 1;
                    }
                }
                Key::RawKey(RawKey::Delete) => {
                    if state.cursor < self.text.len() {
                        self.text.remove(state.cursor);
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

        let cursor_pos = lines_rect.min + lines.char_position(state.cursor);
        let cursor_rect = Rect::new(
            cursor_pos,
            Position::new(cursor_pos.x + 1, cursor_pos.y + F::CHAR_HEIGHT as i64),
        );
        context.fb.draw_rect(cursor_rect, Color::new(255, 255, 255));

        context.state_insert(id, state);

        result
    }
}
