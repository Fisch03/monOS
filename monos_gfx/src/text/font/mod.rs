mod cozette;
pub use cozette::Cozette;

mod glean;
pub use glean::Glean;

mod haeberli;
pub use haeberli::Haeberli;

pub trait Font {
    const CHAR_WIDTH: u32;
    const CHAR_HEIGHT: u32;

    fn get_char(character: char) -> Option<&'static [u8]>;
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
