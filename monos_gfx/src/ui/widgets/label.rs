use super::TextWrap;
use crate::types::*;
use crate::ui::*;
use widgets::Lines;

pub struct Label<'a> {
    text: &'a str,
    wrap: TextWrap,
}

impl<'a> Label<'a> {
    pub fn new(text: &str) -> Label {
        Label {
            text,
            wrap: TextWrap::Disabled,
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Label<'a> {
        self.wrap = wrap;
        self
    }

    pub fn layout(&self, max_dimensions: Dimension) -> Lines<'a> {
        match self.wrap {
            TextWrap::Disabled => Lines::layout_single_line(self.text, max_dimensions.width),
            TextWrap::Enabled { hyphenate } => {
                Lines::layout_wrapped(self.text, hyphenate, max_dimensions)
            }
        }
    }
}

impl UIElement for Label<'_> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_dimensions = Dimension::new(
            context.placer.max_width(),
            context.fb.scaled_dimensions().height,
        );

        let lines = self.layout(max_dimensions);
        let line_dimensions = lines.dimensions();

        let result = context.placer.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        lines.draw(context.fb, lines_rect.min, Color::new(255, 255, 255));

        result
    }
}
