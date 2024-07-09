use super::Font;
use super::TextWrap;
use crate::types::*;
use crate::ui::*;
use core::marker::PhantomData;
use widgets::Lines;

pub struct Label<'a, F>
where
    F: Font,
{
    text: &'a str,
    wrap: TextWrap,
    font: PhantomData<F>,
}

impl<'a, F: Font> Label<'a, F> {
    pub fn new(text: &str) -> Label<F> {
        Label::<F> {
            text,
            wrap: TextWrap::Disabled,
            font: PhantomData,
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Label<'a, F> {
        self.wrap = wrap;
        self
    }
}

impl<F: Font> UIElement for Label<'_, F> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_dimensions =
            Dimension::new(context.placer.max_width(), context.fb.dimensions().height);

        let lines = Lines::<F>::layout(self.text, self.wrap, max_dimensions);
        let line_dimensions = lines.dimensions();

        let result = context.placer.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        lines.draw(context.fb, lines_rect.min, Color::new(255, 255, 255));

        result
    }
}
