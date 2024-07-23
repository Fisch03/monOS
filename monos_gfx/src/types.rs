use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const fn new(x: i64, y: i64) -> Position {
        Position { x, y }
    }

    pub const fn zero() -> Position {
        Position { x: 0, y: 0 }
    }

    pub const fn from_dimensions(dimensions: Dimension) -> Position {
        Position {
            x: dimensions.width as i64,
            y: dimensions.height as i64,
        }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y) as f32
    }
}

impl core::ops::Neg for Position {
    type Output = Position;
    fn neg(self) -> Position {
        Position {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl core::ops::Add<i64> for Position {
    type Output = Position;
    fn add(self, rhs: i64) -> Position {
        Position {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl core::ops::Sub<i64> for Position {
    type Output = Position;
    fn sub(self, rhs: i64) -> Position {
        Position {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl core::ops::Mul<i64> for Position {
    type Output = Position;
    fn mul(self, rhs: i64) -> Position {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl core::ops::Div<i64> for Position {
    type Output = Position;
    fn div(self, rhs: i64) -> Position {
        Position {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl core::ops::Rem<i64> for Position {
    type Output = Position;
    fn rem(self, rhs: i64) -> Position {
        Position {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}

impl core::ops::Add<Position> for Position {
    type Output = Position;
    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl core::ops::Sub<Position> for Position {
    type Output = Position;
    fn sub(self, rhs: Position) -> Position {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl core::ops::Add<Dimension> for Position {
    type Output = Position;
    fn add(self, rhs: Dimension) -> Position {
        Position {
            x: self.x + rhs.width as i64,
            y: self.y + rhs.height as i64,
        }
    }
}

impl core::ops::Sub<Dimension> for Position {
    type Output = Position;
    fn sub(self, rhs: Dimension) -> Position {
        Position {
            x: self.x - rhs.width as i64,
            y: self.y - rhs.height as i64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Rect {
    pub min: Position,
    pub max: Position,
}

impl Rect {
    pub const fn new(min: Position, max: Position) -> Rect {
        Rect { min, max }
    }

    pub const fn zero() -> Rect {
        Rect {
            min: Position::new(0, 0),
            max: Position::new(0, 0),
        }
    }

    pub const fn from_dimensions(dimensions: Dimension) -> Rect {
        Rect {
            min: Position::new(0, 0),
            max: Position::new(dimensions.width as i64, dimensions.height as i64),
        }
    }

    pub const fn centered_in(parent: Rect, dimensions: Dimension) -> Rect {
        let min = Position::new(
            parent.min.x + (parent.width() as i64 - dimensions.width as i64) / 2,
            parent.min.y + (parent.height() as i64 - dimensions.height as i64) / 2,
        );
        let max = Position::new(
            min.x + dimensions.width as i64,
            min.y + dimensions.height as i64,
        );
        Rect { min, max }
    }

    pub const fn dimensions(&self) -> Dimension {
        Dimension::new(self.width(), self.height())
    }

    pub const fn width(&self) -> u32 {
        (self.max.x - self.min.x) as u32
    }

    pub const fn height(&self) -> u32 {
        (self.max.y - self.min.y) as u32
    }

    pub const fn center(&self) -> Position {
        Position {
            x: (self.min.x + self.max.x) / 2,
            y: (self.min.y + self.max.y) / 2,
        }
    }

    pub const fn shrink(&self, padding: u32) -> Rect {
        Rect {
            min: Position::new(self.min.x + padding as i64, self.min.y + padding as i64),
            max: Position::new(self.max.x - padding as i64, self.max.y - padding as i64),
        }
    }

    pub const fn contains(&self, pos: Position) -> bool {
        pos.x >= self.min.x && pos.x < self.max.x && pos.y >= self.min.y && pos.y < self.max.y
    }

    pub const fn translate(&self, offset: Position) -> Rect {
        Rect {
            min: Position {
                x: self.min.x + offset.x,
                y: self.min.y + offset.y,
            },
            max: Position {
                x: self.max.x + offset.x,
                y: self.max.y + offset.y,
            },
        }
    }

    pub const fn intersects(&self, other: &Rect) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// check if self sits on the edge of other
    pub const fn intersects_edge(&self, other: &Rect) -> Option<Edge> {
        if !self.intersects(other) {
            return None;
        }

        if self.min.y < other.min.y && self.max.y > other.min.y {
            return Some(Edge::Top);
        } else if self.max.y > other.max.y && self.min.y < other.max.y {
            return Some(Edge::Bottom);
        } else if self.min.x < other.min.x && self.max.x > other.min.x {
            return Some(Edge::Left);
        } else if self.max.x > other.max.x && self.min.x < other.max.x {
            return Some(Edge::Right);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

impl Dimension {
    pub const fn new(width: u32, height: u32) -> Dimension {
        Dimension { width, height }
    }

    pub const fn zero() -> Dimension {
        Dimension {
            width: 0,
            height: 0,
        }
    }
}

impl core::ops::Add<u32> for Dimension {
    type Output = Dimension;
    fn add(self, rhs: u32) -> Dimension {
        Dimension {
            width: self.width + rhs,
            height: self.height + rhs,
        }
    }
}

impl core::ops::Mul<u32> for Dimension {
    type Output = Dimension;
    fn mul(self, rhs: u32) -> Dimension {
        Dimension {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl core::ops::Div<u32> for Dimension {
    type Output = Dimension;
    fn div(self, rhs: u32) -> Dimension {
        Dimension {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}
