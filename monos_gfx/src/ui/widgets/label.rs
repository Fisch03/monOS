use crate::text::{ColorMode, Font, Origin, TextWrap};
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
    color: ColorMode<'a>,
}

pub struct ScrollableLabel<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
    label: Label<'a, F, I>,
    scroll_x: Option<u32>,
    scroll_y: Option<u32>,
    origin: Origin,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ScrollState {
    offset: Position,
}

impl<'a, F> Label<'a, F, Once<&'a str>>
where
    F: Font,
{
    pub fn new(text: &'a str) -> Label<'a, F, Once<&'a str>> {
        Label::<F, Once<&'a str>> {
            text: once(text),
            wrap: TextWrap::Disabled,
            font: PhantomData,
            color: ColorMode::default(),
        }
    }
}

impl<'a, F, I> Label<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
    pub fn new_iter(text: I) -> Label<'a, F, I> {
        Label::<F, I> {
            text,
            wrap: TextWrap::Disabled,
            font: PhantomData,
            color: ColorMode::default(),
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Label<'a, F, I> {
        self.wrap = wrap;
        self
    }

    pub fn text_color(mut self, color: Color) -> Label<'a, F, I> {
        self.color = ColorMode::Single(color);
        self
    }

    pub fn text_colors(mut self, colors: &'a [Color]) -> Label<'a, F, I> {
        self.color = ColorMode::PerLine(colors);
        self
    }
}

impl<'a, F, I> UIElement for Label<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
    fn draw(self, context: &mut UIContext) -> UIResult {
        let max_dimensions = Dimension::new(
            context.placer.max_width(),
            context
                .fb
                .as_ref()
                .map(|fb| fb.dimensions().height)
                .unwrap_or(u32::MAX),
        );

        let lines = Lines::<F>::layout_iter(self.text, self.wrap, max_dimensions);
        let line_dimensions = lines.dimensions();

        let result = context.alloc_space(line_dimensions);
        let lines_rect = Rect::centered_in(result.rect, line_dimensions);

        if let Some(fb) = &mut context.fb {
            lines.draw(*fb, lines_rect.min, self.color);
        }

        result
    }
}

impl<'a, F> ScrollableLabel<'a, F, Once<&'a str>>
where
    F: Font,
{
    pub fn new(text: &'a str, origin: Origin) -> ScrollableLabel<'a, F, Once<&'a str>> {
        ScrollableLabel::<F, Once<&'a str>> {
            label: Label::new(text),
            scroll_x: None,
            scroll_y: None,
            origin,
        }
    }
}

impl<'a, F, I> ScrollableLabel<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
    pub fn new_iter(text: I, origin: Origin) -> ScrollableLabel<'a, F, I> {
        ScrollableLabel::<F, I> {
            label: Label::new_iter(text),
            scroll_x: None,
            scroll_y: None,
            origin,
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
        self.label = self.label.wrap(wrap);
        self
    }

    pub fn text_color(mut self, color: Color) -> ScrollableLabel<'a, F, I> {
        self.label = self.label.text_color(color);
        self
    }

    pub fn text_colors(mut self, colors: &'a [Color]) -> ScrollableLabel<'a, F, I> {
        self.label = self.label.text_colors(colors);
        self
    }
}

impl<'a, F, I> UIElement for ScrollableLabel<'a, F, I>
where
    F: Font,
    I: Iterator<Item = &'a str>,
{
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
            .min(
                context
                    .fb
                    .as_ref()
                    .map(|fb| fb.dimensions().height)
                    .unwrap_or(u32::MAX),
            ),
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

        let lines = Lines::<F>::layout_iter(self.label.text, self.label.wrap, max_text_dimensions);
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

        if let Some(fb) = &mut context.fb {
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

                fb.draw_vert_line(
                    Position::new(lines_rect.max.x - 1, scroll_y),
                    scroll_len,
                    Color::new(255, 255, 255),
                );
            }

            lines.draw_clipped(*fb, lines_rect, state.offset, self.origin, self.label.color);
        }

        context.state_insert(id, state);

        result
    }
}
