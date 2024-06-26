use crate::{input::Input, Dimension, Framebuffer, Position, Rect};
pub mod widgets;

pub trait UIElement {
    fn draw(self, context: &mut UIContext) -> UIResult;
}

#[derive(Debug)]
pub struct UIContext<'a> {
    placer: Placer,
    fb: &'a mut Framebuffer,
}

impl UIContext<'_> {
    pub fn add(&mut self, element: impl UIElement) -> UIResult {
        element.draw(self)
    }

    pub fn label(&mut self, text: &str) -> UIResult {
        self.add(widgets::Label::new(text))
    }
    pub fn button(&mut self, text: &str) -> UIResult {
        self.add(widgets::Button::new(text))
    }

    pub fn margin(&mut self, mode: MarginMode) {
        self.placer.margin_mode = mode;
    }
    pub fn padding(&mut self, mode: PaddingMode) {
        self.placer.padding_mode = mode;
    }
    pub fn gap(&mut self, gap: u32) {
        self.placer.gap = gap;
    }
}

pub struct UIFrame {
    direction: Direction,
}

impl UIFrame {
    pub fn new(direction: Direction) -> UIFrame {
        UIFrame { direction }
    }

    pub fn draw_frame<F>(&mut self, fb: &mut Framebuffer, area: Rect, input: &Input, f: F)
    where
        F: FnOnce(&mut UIContext),
    {
        let mut context = UIContext {
            placer: Placer::new(area, self.direction),
            fb,
        };

        f(&mut context);
    }
}

#[derive(Debug)]
pub struct Placer {
    max_rect: Rect,

    cursor: Position,
    cross_size: u32,

    direction: Direction,

    padding_mode: PaddingMode,
    margin_mode: MarginMode,
    gap: u32,
}

/// affects how big the total space allocated for a widget will be.
///
/// minimum: the widget will be allocated the exact space it needs.
/// grow: the widget will fill the entire cross axis.
/// at_least: the widget will be allocated at least the specified size on the cross axis.
#[derive(Debug)]
pub enum MarginMode {
    Minimum,
    Grow,
    // AtLeast(u32), // sorta broken right now, i'll fix it once i actually need this
}

/// affects how much of the allocated space (determined by the margin mode) will be filled with the widget.
///
/// minimum: the widget will only fill the exact space it needs
/// fill: the widget will fill the entire space allocated for it.
#[derive(Debug)]
pub enum PaddingMode {
    Fill,
    Gap(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    // BottomToTop, // too lazy. i dont think i'll need it anyway
}

impl Placer {
    fn new(bounds: Rect, direction: Direction) -> Self {
        let cursor = match direction {
            Direction::LeftToRight => Position::new(bounds.min.x, bounds.min.y),
            Direction::RightToLeft => Position::new(bounds.max.x, bounds.min.y),
            Direction::TopToBottom => Position::new(bounds.min.x, bounds.min.y),
            // Direction::BottomToTop => Position::new(bounds.min.x, bounds.max.y),
        };

        Self {
            max_rect: bounds,
            direction,
            cross_size: 0,
            cursor,
            margin_mode: MarginMode::Minimum,
            padding_mode: PaddingMode::Gap(2),
            gap: 1,
        }
    }

    pub fn max_width(&self) -> u32 {
        let max_incl_gap = match self.direction {
            Direction::LeftToRight | Direction::TopToBottom => self.max_rect.max.x - self.cursor.x,
            Direction::RightToLeft => self.cursor.x - self.max_rect.min.x,
        } - match self.margin_mode {
            // MarginMode::AtLeast(min_width) => min_width as i64 / 2,
            _ => 0,
        };

        let total_gap = self.gap * 2
            + match self.padding_mode {
                PaddingMode::Gap(gap) => gap * 2,
                PaddingMode::Fill => 0,
            };

        let max = max_incl_gap - total_gap as i64;

        if max < 0 {
            return 0;
        }
        return max as u32;
    }

    /// allocate a rect of the desired space.
    ///
    /// there is no guarantee that the returned rect will fit the desired space.
    pub fn alloc_space(&mut self, mut desired_space: Dimension) -> UIResult {
        let total_gap = self.gap * 2
            + match self.padding_mode {
                PaddingMode::Gap(gap) => gap * 2,
                PaddingMode::Fill => 0,
            };

        desired_space.height += total_gap;
        desired_space.width += total_gap;

        let mut padded_space = Rect::zero();

        match self.direction {
            Direction::LeftToRight => {
                padded_space.min = self.cursor;
                padded_space.max = match self.margin_mode {
                    MarginMode::Minimum => Position::new(
                        self.cursor.x + desired_space.width as i64,
                        self.cursor.y + desired_space.height as i64,
                    ),
                    MarginMode::Grow => Position::new(
                        self.cursor.x + desired_space.width as i64,
                        self.cursor.y + self.max_rect.height() as i64,
                    ),
                    // MarginMode::AtLeast(min_height) => Position::new(
                    //     self.cursor.x + desired_space.width as i64,
                    //     (min_height as i64).max(self.cursor.y + desired_space.height as i64),
                    // ),
                };
                self.cursor.x += desired_space.width as i64;
                if self.cursor.x >= self.max_rect.max.x {
                    self.cursor.x = self.max_rect.min.x;
                    self.cursor.y += self.cross_size as i64;
                    self.cross_size = 0;
                }
            }
            Direction::RightToLeft => {
                padded_space.min =
                    Position::new(self.cursor.x - desired_space.width as i64, self.cursor.y);
                padded_space.max = match self.margin_mode {
                    MarginMode::Minimum => {
                        Position::new(self.cursor.x, self.cursor.y + desired_space.height as i64)
                    }
                    MarginMode::Grow => {
                        Position::new(self.cursor.x, self.cursor.y + desired_space.height as i64)
                    } // MarginMode::AtLeast(min_height) => Position::new(
                      //     self.cursor.x,
                      //     (min_height as i64).max(self.cursor.y + desired_space.height as i64),
                      // ),
                };
                self.cursor.x -= desired_space.width as i64;
                if self.cursor.x <= self.max_rect.min.x {
                    self.cursor.x = self.max_rect.max.x;
                    self.cursor.y -= self.cross_size as i64;
                    self.cross_size = 0;
                }
            }
            Direction::TopToBottom => {
                padded_space.min = self.cursor;
                padded_space.max = match self.margin_mode {
                    MarginMode::Minimum => Position::new(
                        self.cursor.x + desired_space.width as i64,
                        self.cursor.y + desired_space.height as i64,
                    ),
                    MarginMode::Grow => Position::new(
                        self.max_rect.max.x,
                        self.cursor.y + desired_space.height as i64,
                    ),
                    // MarginMode::AtLeast(min_width) => Position::new(
                    //     (min_width as i64).max(self.cursor.x + desired_space.width as i64),
                    //     self.cursor.y + desired_space.height as i64,
                    // ),
                };
                self.cursor.y += desired_space.height as i64;
                if self.cursor.y >= self.max_rect.max.y {
                    self.cursor.y = self.max_rect.min.y;
                    self.cursor.x += self.cross_size as i64;
                    self.cross_size = 0;
                }
            }
        }

        match self.direction {
            Direction::LeftToRight | Direction::RightToLeft => {
                self.cross_size = self.cross_size.max(padded_space.height() as u32)
            }
            Direction::TopToBottom => {
                self.cross_size = self.cross_size.max(padded_space.width() as u32)
            }
        };

        let mut widget_space = match self.padding_mode {
            PaddingMode::Gap(_) => {
                let center = padded_space.min + (padded_space.max - padded_space.min) / 2;
                let min = center - (desired_space / 2);
                let max = center + (desired_space / 2);

                Rect::new(min, max)
            }
            PaddingMode::Fill => padded_space,
        };
        widget_space.min.x += self.gap as i64;
        widget_space.min.y += self.gap as i64;
        widget_space.max.x -= self.gap as i64;
        widget_space.max.y -= self.gap as i64;

        UIResult {
            rect: widget_space,
            full_rect: padded_space,

            clicked: false,
            hovered: false,
        }
    }
}

#[derive(Debug)]
pub struct UIResult {
    // rect of the widget
    pub rect: Rect,
    // rect of the widget including its margin
    pub full_rect: Rect,

    // whether the widget was clicked
    pub clicked: bool,
    // whether the mouse is hovering over the widget
    pub hovered: bool,
}

impl UIResult {
    pub const fn empty() -> Self {
        Self {
            rect: Rect::zero(),
            full_rect: Rect::zero(),
            clicked: false,
            hovered: false,
        }
    }

    pub fn set_clicked(&mut self, clicked: bool) {
        self.clicked = clicked;
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
    }
}
