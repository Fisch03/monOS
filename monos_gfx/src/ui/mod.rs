use crate::{input::Input, text::Font, Dimension, Framebuffer, Position, Rect};
pub mod widgets;
pub use crate::text::{Lines, TextWrap};

use core::any::TypeId;
use hashbrown::hash_map::HashMap;
use rustc_hash::{FxBuildHasher, FxHasher};
pub use serde::{Deserialize, Serialize};

pub trait UIElement {
    fn draw(self, context: &mut UIContext) -> UIResult;
}

#[derive(Debug)]
pub struct UIContext<'a, 'fb> {
    pub placer: Placer,
    pub fb: &'a mut Framebuffer<'fb>,
    pub input: &'a mut Input,
    state: &'a mut Option<UIStateMap>,
    auto_id_source: u32,
}

impl UIContext<'_, '_> {
    pub fn add(&mut self, element: impl UIElement) -> UIResult {
        element.draw(self)
    }

    pub fn alloc_space(&mut self, desired_space: Dimension) -> UIResult {
        let mut result = self.placer.alloc_space(desired_space);
        if result.rect.contains(self.input.mouse.position) {
            result.set_hovered(true);
            if self.input.mouse.left_button.clicked {
                result.set_clicked(true);
            }
        }

        result
    }

    pub fn next_id(&mut self) -> u64 {
        use core::hash::Hasher;

        let mut hasher = FxHasher::default();
        self.auto_id_source += 1;
        hasher.write_u64(self.auto_id_source as u64);
        hasher.finish()
    }

    pub fn next_id_from_string(&mut self, string: &str) -> u64 {
        use core::hash::Hasher;

        let mut hasher = FxHasher::default();
        hasher.write(string.as_bytes());
        self.auto_id_source += 1;
        // hasher.write_u64(self.auto_id_source as u64);
        hasher.finish()
    }

    pub fn state_get<T: UIState>(&mut self, key: u64) -> Option<T> {
        match self.state {
            Some(ref mut state) => state.get(key),
            None => None,
        }
    }

    pub fn state_insert<T: UIState>(&mut self, key: u64, value: T) {
        match self.state {
            Some(ref mut state) => state.insert(key, value),
            None => (),
        }
    }

    // widget shortcuts
    pub fn label<'a, F: Font>(&mut self, text: &'a str) -> UIResult {
        self.add(widgets::Label::<F, _>::new(text))
    }
    pub fn textbox<F: Font>(&mut self, text: &mut String) -> UIResult {
        self.add(widgets::Textbox::<F>::new(text))
    }
    pub fn button<F: Font>(&mut self, text: &str) -> UIResult {
        self.add(widgets::Button::<F>::new(text))
    }
    pub fn img_button(&mut self, image: &crate::Image) -> UIResult {
        self.add(widgets::ImageButton::new(image))
    }

    // layout
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

pub trait UIState
where
    Self: core::fmt::Debug + Clone + Serialize + for<'de> Deserialize<'de> + 'static,
{
}

impl<T> UIState for T where
    T: core::fmt::Debug + Clone + Serialize + for<'de> Deserialize<'de> + 'static
{
}

#[derive(Debug, Clone)]
pub struct UIStateMap {
    content: HashMap<(u64, TypeId), Vec<u8>, FxBuildHasher>,
}

impl UIStateMap {
    pub fn new() -> Self {
        Self {
            content: HashMap::with_hasher(FxBuildHasher::default()),
        }
    }

    pub fn insert<T: UIState>(&mut self, key: u64, state: T) {
        self.content.insert(
            (key, TypeId::of::<T>()),
            postcard::to_allocvec(&state).unwrap(),
        );
    }

    pub fn get<T: UIState>(&self, key: u64) -> Option<T> {
        self.content
            .get(&(key, TypeId::of::<T>()))
            .map(|data| postcard::from_bytes(data).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct UIFrame {
    direction: Direction,
    state: Option<UIStateMap>,
}

impl UIFrame {
    pub fn new(direction: Direction) -> UIFrame {
        UIFrame {
            direction,
            state: Some(UIStateMap::new()),
        }
    }

    // if you already know you wont be storing any state and performance is a concern, you can use
    // this (for example if you only need to render a single label)
    pub fn new_stateless(direction: Direction) -> UIFrame {
        UIFrame {
            direction,
            state: None,
        }
    }

    pub fn draw_frame<F>(&mut self, fb: &mut Framebuffer<'_>, area: Rect, input: &mut Input, f: F)
    where
        F: FnOnce(&mut UIContext),
    {
        let mut context = UIContext {
            placer: Placer::new(area, self.direction),
            fb,
            input,
            auto_id_source: 0,
            state: &mut self.state,
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
/// fixed: put the widget at a fixed position and size.
/// at_least: the widget will be allocated at least the specified size on the cross axis.
#[derive(Debug)]
pub enum MarginMode {
    Minimum,
    Grow,
    Fixed(Position, Dimension),
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
    BottomToTop,
}

impl Placer {
    fn new(bounds: Rect, direction: Direction) -> Self {
        let cursor = match direction {
            Direction::LeftToRight => Position::new(bounds.min.x, bounds.min.y),
            Direction::RightToLeft => Position::new(bounds.max.x, bounds.min.y),
            Direction::TopToBottom => Position::new(bounds.min.x, bounds.min.y),
            Direction::BottomToTop => Position::new(bounds.min.x, bounds.max.y),
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
            Direction::LeftToRight | Direction::TopToBottom | Direction::BottomToTop => {
                self.max_rect.max.x - self.cursor.x
            }
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
    fn alloc_space(&mut self, mut desired_space: Dimension) -> UIResult {
        if let MarginMode::Fixed(position, dimension) = self.margin_mode {
            return UIResult {
                rect: Rect::new(position, position + dimension),
                full_rect: Rect::new(position, position + dimension),
                ..Default::default()
            };
        }

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
                    MarginMode::Fixed(_, _) => unreachable!(),
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
                    }
                    MarginMode::Fixed(_, _) => unreachable!(),
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
                    MarginMode::Fixed(_, _) => unreachable!(),
                };
                self.cursor.y += desired_space.height as i64;
                if self.cursor.y >= self.max_rect.max.y {
                    self.cursor.y = self.max_rect.min.y;
                    self.cursor.x += self.cross_size as i64;
                    self.cross_size = 0;
                }
            }
            Direction::BottomToTop => {
                padded_space.min =
                    Position::new(self.cursor.x, self.cursor.y - desired_space.height as i64);
                padded_space.max = match self.margin_mode {
                    MarginMode::Minimum => {
                        Position::new(self.cursor.x + desired_space.width as i64, self.cursor.y)
                    }
                    MarginMode::Grow => Position::new(self.max_rect.max.x, self.cursor.y),
                    MarginMode::Fixed(_, _) => unreachable!(),
                };
                self.cursor.y -= desired_space.height as i64;
                if self.cursor.y <= self.max_rect.min.y {
                    self.cursor.y = self.max_rect.max.y;
                    self.cursor.x += self.cross_size as i64;
                    self.cross_size = 0;
                }
            }
        }

        match self.direction {
            Direction::LeftToRight | Direction::RightToLeft => {
                self.cross_size = self.cross_size.max(padded_space.height() as u32)
            }
            Direction::TopToBottom | Direction::BottomToTop => {
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

            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct UIResult {
    // unique id of the widget. this will only be set if the widget stored some state or is focusable
    pub id: Option<u64>,

    // rect of the widget
    pub rect: Rect,
    // rect of the widget including its margin
    pub full_rect: Rect,

    // whether the widget was clicked
    pub clicked: bool,
    // whether the mouse is hovering over the widget
    pub hovered: bool,

    // the exact meaning of this field is up to the widget.
    // a textbox might set this to true if the user pressed enter.
    pub submitted: bool,
}

impl UIResult {
    pub fn set_clicked(&mut self, clicked: bool) {
        self.clicked = clicked;
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.hovered = hovered;
    }
}
