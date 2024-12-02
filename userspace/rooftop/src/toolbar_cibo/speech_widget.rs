use monos_gfx::{
    text::font,
    ui::{Deserialize, Lines, Serialize, TextWrap, UIContext, UIElement, UIResult},
    Color, Dimension, Position, Rect,
};

#[derive(Debug, Clone)]
pub struct SpeechWidget<'a> {
    text: &'a str,
    custom_id: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatWidgetState {
    size: u32,
    open: bool,
}

impl<'a> SpeechWidget<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            custom_id: None,
        }
    }

    pub fn with_id(text: &'a str, id: &'a str) -> Self {
        Self {
            text,
            custom_id: Some(id),
        }
    }
}

impl UIElement for SpeechWidget<'_> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let id = if let Some(id) = self.custom_id {
            context.next_id_from_string(id)
        } else {
            context.next_id_from_string(self.text)
        };

        let mut state: ChatWidgetState = context.state_get(id).unwrap_or_default();

        let line_max_dimensions = Dimension::new(
            context.placer.max_width() - 2,
            context
                .fb
                .as_ref()
                .map(|fb| fb.dimensions().height)
                .unwrap_or(u32::MAX),
        );
        let lines = Lines::<font::Glean>::layout(
            self.text,
            TextWrap::Enabled { hyphenate: true },
            line_max_dimensions,
        );
        let line_dimensions = lines.dimensions();

        let dimensions = Dimension::new(line_dimensions.width + 2, line_dimensions.height + 4);

        let mut result = context.alloc_space(dimensions);
        result.rect.max.y -= 2;

        let center_x = result.rect.center().x;

        let drawn_rect = if state.open {
            result.rect
        } else {
            state.size += 3;
            let width = result.rect.width().min(state.size);
            let height = result.rect.height().min(state.size);

            if width == result.rect.width() && height == result.rect.height() {
                state.open = true;
            }

            Rect::new(
                Position::new(
                    center_x - width as i64 / 2,
                    result.rect.max.y - height as i64,
                ),
                Position::new(center_x + width as i64 / 2, result.rect.max.y),
            )
        };

        // TODO: horribleness. add line drawing functions
        if let Some(fb) = &mut context.fb {
            let inner_rect = drawn_rect.shrink(1);
            fb.draw_rect(inner_rect, Color::new(255, 255, 255));
            let stem_rect = Rect::new(
                Position::new(center_x - 2, drawn_rect.max.y - 1),
                Position::new(center_x + 2, drawn_rect.max.y + 1),
            );
            fb.draw_rect(stem_rect, Color::new(255, 255, 255));

            let upper_line = Rect::new(
                Position::new(drawn_rect.min.x + 1, drawn_rect.min.y),
                Position::new(drawn_rect.max.x - 1, drawn_rect.min.y + 1),
            );
            fb.draw_rect(upper_line, Color::new(0, 0, 0));

            let lower_line_left = Rect::new(
                Position::new(drawn_rect.min.x + 1, drawn_rect.max.y - 1),
                Position::new(center_x - 2, drawn_rect.max.y),
            );
            let lower_line_right = Rect::new(
                Position::new(center_x + 2, drawn_rect.max.y - 1),
                Position::new(drawn_rect.max.x - 1, drawn_rect.max.y),
            );
            fb.draw_rect(lower_line_left, Color::new(0, 0, 0));
            fb.draw_rect(lower_line_right, Color::new(0, 0, 0));

            fb.draw_pixel(
                Position::new(center_x - 2, drawn_rect.max.y),
                Color::new(0, 0, 0),
            );
            fb.draw_pixel(
                Position::new(center_x - 1, drawn_rect.max.y + 1),
                Color::new(0, 0, 0),
            );
            fb.draw_pixel(
                Position::new(center_x, drawn_rect.max.y + 1),
                Color::new(0, 0, 0),
            );
            fb.draw_pixel(
                Position::new(center_x + 1, drawn_rect.max.y),
                Color::new(0, 0, 0),
            );

            let left_line = Rect::new(
                Position::new(drawn_rect.min.x, drawn_rect.min.y + 1),
                Position::new(drawn_rect.min.x + 1, drawn_rect.max.y - 1),
            );
            fb.draw_rect(left_line, Color::new(0, 0, 0));

            let right_line = Rect::new(
                Position::new(drawn_rect.max.x - 1, drawn_rect.min.y + 1),
                Position::new(drawn_rect.max.x, drawn_rect.max.y - 1),
            );
            fb.draw_rect(right_line, Color::new(0, 0, 0));

            if state.open {
                let lines_rect = Rect::centered_in(result.rect, line_dimensions);
                lines.draw(fb, lines_rect.min, Color::new(0, 0, 0));
            }
        }

        context.state_insert(id, state);

        result
    }
}
