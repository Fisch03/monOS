use crate::text::{Font, Origin, TextWrap};
use crate::types::*;
use crate::ui::*;
use core::iter::{once, Once};
use core::marker::PhantomData;

pub struct Label<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
    text: I,
    wrap: TextWrap,
    font: PhantomData<F>,
    color: Color,
}

pub struct ScrollableLabel<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
    text: I,
    wrap: TextWrap,
    scroll_x: Option<u32>,
    scroll_y: Option<u32>,
    origin: Origin,
    font: PhantomData<F>,
    color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ScrollState {
    offset: Position,
}

impl<'a, F: Font> Label<'a, F, Once<&'a str>> {
    pub fn new(text: &'a str) -> Label<F, Once<&'a str>> {
        Label::<F, Once<&'a str>> {
            text: once(text),
            wrap: TextWrap::Disabled,
            font: PhantomData,
            color: Color::new(255, 255, 255),
        }
    }
}

impl<'a, F: Font, I: Iterator<Item = &'a str>> Label<'a, F, I> {
    pub fn new_iter(text: I) -> Label<'a, F, I> {
        Label::<F, I> {
            text,
            wrap: TextWrap::Disabled,
            font: PhantomData,
            color: Color::new(255, 255, 255),
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Label<'a, F, I> {
        self.wrap = wrap;
        self
    }

    pub fn text_color(mut self, color: Color) -> Label<'a, F, I> {
        self.color = color;
        self
    }
}

impl<'a, F: Font, I: Iterator<Item = &'a str>> UIElement for Label<'a, F, I> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_dimensions =
            Dimension::new(context.placer.max_width(), context.fb.dimensions().height);

        let lines = Lines::<F>::layout_iter(self.text, self.wrap, max_dimensions);
        let line_dimensions = lines.dimensions();

        let result = context.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        lines.draw(context.fb, lines_rect.min, self.color);

        result
    }
}

impl<'a, F: Font> ScrollableLabel<'a, F, Once<&'a str>> {
    pub fn new(text: &'a str, origin: Origin) -> ScrollableLabel<F, Once<&'a str>> {
        ScrollableLabel::<F, Once<&'a str>> {
            text: once(text),
            wrap: TextWrap::Enabled { hyphenate: false },
            scroll_x: None,
            scroll_y: None,
            origin,
            color: Color::new(255, 255, 255),
            font: PhantomData,
        }
    }
}

impl<'a, F: Font, I: Iterator<Item = &'a str>> ScrollableLabel<'a, F, I> {
    pub fn new_iter(text: I, origin: Origin) -> ScrollableLabel<'a, F, I> {
        ScrollableLabel::<F, I> {
            text,
            wrap: TextWrap::Enabled { hyphenate: false },
            scroll_x: None,
            scroll_y: None,
            origin,
            color: Color::new(255, 255, 255),
            font: PhantomData,
        }
    }

    pub fn scroll_x(mut self, x: u32) -> ScrollableLabel<'a, F, I> {
        self.scroll_x = Some(x);
        self
    }

    pub fn scroll_y(mut self, y: u32) -> ScrollableLabel<'a, F, I> {
        self.scroll_y = Some(y);
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> ScrollableLabel<'a, F, I> {
        self.wrap = wrap;
        self
    }

    pub fn text_color(mut self, color: Color) -> ScrollableLabel<'a, F, I> {
        self.color = color;
        self
    }
}

impl<'a, F: Font, I: Iterator<Item = &'a str>> UIElement for ScrollableLabel<'a, F, I> {
    fn draw(self, context: &mut UIContext) -> UIResult {
        let id = context.next_id();
        let mut state: ScrollState = context.state_get(id).unwrap_or_default();

        let max_dimensions = Dimension::new(
            if let Some(x) = self.scroll_x {
                x
            } else {
                u32::MAX
            }
            .min(context.placer.max_width()),
            if let Some(y) = self.scroll_y {
                y
            } else {
                u32::MAX
            }
            .min(context.fb.dimensions().height),
        );

        let max_text_dimensions = Dimension::new(
            if self.scroll_x.is_none() {
                max_dimensions.width
            } else {
                u32::MAX as u32
            },
            if self.scroll_y.is_none() {
                max_dimensions.height
            } else {
                u32::MAX as u32
            },
        );

        let lines = Lines::<F>::layout_iter(self.text, self.wrap, max_text_dimensions);
        let line_dimensions = lines.dimensions();
        let result = context.alloc_space(max_dimensions);

        if result.hovered {
            match self.origin {
                Origin::Top => {
                    state.offset.y += context.input.mouse.scroll;
                }
                Origin::Bottom => {
                    state.offset.y -= context.input.mouse.scroll;
                }
            }
        }

        let lines_rect = match self.origin {
            Origin::Top => Rect::new(
                result.rect.min,
                Position::new(
                    result.rect.max.x,
                    result.rect.min.y + max_dimensions.height as i64,
                ),
            ),
            Origin::Bottom => Rect::new(
                Position::new(
                    result.rect.min.x,
                    result.rect.max.y - max_dimensions.height as i64,
                ),
                result.rect.max,
            ),
        };

        state.offset = Position::new(
            state
                .offset
                .x
                .min(line_dimensions.width as i64 - lines_rect.width() as i64)
                .max(0),
            state
                .offset
                .y
                .min(line_dimensions.height as i64 - lines_rect.height() as i64)
                .max(0),
        );

        if result.hovered && line_dimensions.height > lines_rect.height() as u32 {
            let scroll_pct = state.offset.y as f32 / line_dimensions.height as f32;

            let lines_height = lines_rect.height() as f32;
            let scroll_len = (lines_height as f32
                * (lines_height as f32 / line_dimensions.height as f32))
                as i64;

            let scroll_y = (lines_rect.height() as f32 * scroll_pct) as i64;
            let scroll_y = match self.origin {
                Origin::Top => lines_rect.min.y + scroll_y,
                Origin::Bottom => lines_rect.max.y - scroll_y - scroll_len,
            };

            context.fb.draw_vert_line(
                Position::new(lines_rect.max.x - 1, scroll_y),
                scroll_len,
                self.color,
            );
        }
        lines.draw_clipped(
            context.fb,
            lines_rect,
            state.offset,
            self.origin,
            self.color,
        );

        context.state_insert(id, state);

        result
    }
}
