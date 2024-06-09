#[derive(Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }
}

impl core::ops::Mul<usize> for &Position {
    type Output = Position;
    fn mul(self, rhs: usize) -> Position {
        Position {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Debug)]
pub struct Dimension {
    pub width: usize,
    pub height: usize,
}

impl Dimension {
    pub fn new(width: usize, height: usize) -> Dimension {
        Dimension { width, height }
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

pub struct Character {
    pub width: usize,
    pub height: usize,
    pub data: &'static [u8],
}

impl Character {
    pub fn from_raw(raw: &'static [u8]) -> Character {
        let width = usize::from(raw[0]);
        let height = usize::from(raw[1]);
        let data = &raw[2..];

        Self {
            width,
            height,
            data,
        }
    }
}
