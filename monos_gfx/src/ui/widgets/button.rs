use crate::types::*;
use crate::ui::*;
use crate::Image;
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

        let result = context.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        let bg_color = if result.hovered {
            Color::new(200, 200, 200)
        } else {
            Color::new(255, 255, 255)
        };

        context.fb.draw_rect(&result.rect, &bg_color);
        lines.draw(context.fb, lines_rect.min, Color::new(0, 0, 0));

        result
    }
}

pub struct ImageButton<'a> {
    image: &'a Image,
}

impl ImageButton<'_> {
    pub fn new(image: &Image) -> ImageButton {
        ImageButton { image }
    }
}

impl UIElement for ImageButton<'_> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_width = context.placer.max_width();

        let result = context.alloc_space(self.image.dimensions());
        let image_rect = Rect::centered_in(result.rect, self.image.dimensions());

        context.fb.draw_img(&self.image, &image_rect.min);

        result
    }
}
