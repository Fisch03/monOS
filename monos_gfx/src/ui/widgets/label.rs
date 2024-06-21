use crate::fonts::{Cozette, Font};
use crate::types::*;
use crate::ui::*;

pub struct Label<'a> {
    text: &'a str,
}

impl Label<'_> {
    pub fn new(text: &str) -> Label {
        Label { text }
    }
}

impl UIElement for Label<'_> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_width = context.placer.max_width();
        let chars_per_line = max_width / Cozette::CHAR_WIDTH;

        let mut rows = 1;
        let mut column = 0;
        for character in self.text.chars() {
            column += 1;

            if character == '\n' || column >= chars_per_line {
                rows += 1;
                column = 0;
            }
        }

        let dimensions = Dimension {
            width: chars_per_line * Cozette::CHAR_WIDTH,
            height: rows * Cozette::CHAR_HEIGHT,
        };

        let rect = context.placer.alloc_space(dimensions);

        let mut position = rect.min;
        for character in self.text.chars() {
            if character == '\n' || position.x + Cozette::CHAR_WIDTH as i64 > rect.max.x {
                position.x = rect.min.x;
                position.y += Cozette::CHAR_HEIGHT as i64;
                continue;
            }

            context
                .fb
                .draw_char::<Cozette>(&Color::new(255, 255, 255), character, &position);

            position.x += Cozette::CHAR_WIDTH as i64;
        }

        UIResult { rect }
    }
}
