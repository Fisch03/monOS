use crate::{Dimension, Framebuffer, Position, Rect};
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
}

pub struct UIFrame {
    direction: Direction,
}

impl UIFrame {
    pub fn new(direction: Direction) -> UIFrame {
        UIFrame { direction }
    }

    pub fn draw_frame<F>(&mut self, fb: &mut Framebuffer, area: Rect, f: F)
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
    cursor: i64,

    direction: Direction,
}

impl Placer {
    fn new(bounds: Rect, direction: Direction) -> Self {
        let cursor = match direction {
            Direction::LeftToRight => bounds.min.x,
            Direction::RightToLeft => bounds.max.x,
            Direction::TopToBottom => bounds.min.y,
            Direction::BottomToTop => bounds.max.y,
        };

        Self {
            max_rect: bounds,
            direction,
            cursor,
        }
    }

    pub fn max_width(&self) -> u32 {
        (self.max_rect.max.x - self.max_rect.min.x).abs() as u32
    }

    /// allocate a rect of the desired space.
    ///
    /// there is no guarantee that the returned rect will fit the desired space.
    pub fn alloc_space(&mut self, desired_space: Dimension) -> Rect {
        let mut space = Rect::zero();

        match self.direction {
            Direction::LeftToRight => {
                space.min = Position::new(self.cursor, self.max_rect.min.y);
                space.max = Position::new(
                    self.cursor + desired_space.width as i64,
                    self.max_rect.max.y,
                );
                self.cursor += desired_space.width as i64;
            }
            Direction::RightToLeft => {
                space.min = Position::new(
                    self.cursor - desired_space.width as i64,
                    self.max_rect.min.y,
                );
                space.max = Position::new(self.cursor, self.max_rect.max.y);
                self.cursor -= desired_space.width as i64;
            }
            Direction::TopToBottom => {
                space.min = Position::new(self.max_rect.min.x, self.cursor);
                space.max = Position::new(
                    self.max_rect.max.x,
                    self.cursor + desired_space.height as i64,
                );
                self.cursor += desired_space.height as i64;
            }
            Direction::BottomToTop => {
                space.min = Position::new(
                    self.max_rect.min.x,
                    self.cursor - desired_space.height as i64,
                );
                space.max = Position::new(self.max_rect.max.x, self.cursor);
                self.cursor -= desired_space.height as i64;
            }
        }

        space
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Debug)]
pub struct UIResult {
    rect: Rect,
}
