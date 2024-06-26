use crate::types::*;
use crate::ui::*;
use widgets::Lines;

pub struct Button<'a> {
    text: &'a str,
}

impl Button<'_> {
    pub fn new(text: &str) -> Button {
        Button { text }
    }
}

impl UIElement for Button<'_> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_width = context.placer.max_width();

        let lines = Lines::layout_single_line(self.text, max_width);
        if lines.dimensions == Dimension::zero() {
            return UIResult::empty();
        }

        let line_dimensions = lines.dimensions();

        let result = context.placer.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        context
            .fb
            .draw_rect(&result.rect, &Color::new(255, 255, 255));
        lines.draw(context.fb, lines_rect.min, Color::new(0, 0, 0));

        result
    }
}
