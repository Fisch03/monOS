use crate::text::{Font, Lines, TextWrap};
use crate::types::*;
use crate::ui::*;
use crate::Image;
use core::marker::PhantomData;

pub struct Button<'a, F>
where
    F: Font,
{
    text: &'a str,
    font: PhantomData<F>,
}

impl<F: Font> Button<'_, F> {
    pub fn new(text: &str) -> Button<F> {
        Button {
            text,
            font: PhantomData,
        }
    }
}

impl<F: Font> UIElement for Button<'_, F> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_width = context.placer.max_width();

        let lines = Lines::<F>::layout(
            self.text,
            TextWrap::Disabled,
            Dimension::new(max_width, u32::MAX),
        );
        if lines.dimensions() == Dimension::zero() {
            return UIResult::default();
        }

        let line_dimensions = lines.dimensions();

        let result = context.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        let bg_color = if result.hovered {
            Color::new(200, 200, 200)
        } else {
            Color::new(255, 255, 255)
        };

        if let Some(fb) = &mut context.fb {
            fb.draw_rect(result.rect, bg_color);
            lines.draw(*fb, lines_rect.min, Color::new(0, 0, 0));
        }

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
        let result = context.alloc_space(self.image.dimensions());
        let image_rect = Rect::centered_in(result.rect, self.image.dimensions());

        if let Some(fb) = &mut context.fb {
            fb.draw_img(&self.image, image_rect.min);
        }

        result
    }
}
