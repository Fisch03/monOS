#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub const fn new(x: i64, y: i64) -> Position {
        Position { x, y }
    }
}

impl core::ops::Mul<i64> for &Position {
    type Output = Position;
    fn mul(self, rhs: i64) -> Position {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn dimensions(&self) -> Dimension {
        Dimension::new(
            (self.max.x - self.min.x) as u32,
            (self.max.y - self.min.y) as u32,
        )
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
