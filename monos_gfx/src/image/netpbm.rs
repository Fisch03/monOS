use crate::{Color, Dimension};

use super::{Image, ImageFormat, ImageLoader};

pub struct PPMLoader;

impl ImageLoader for PPMLoader {
    fn is_supported(&self, data: &[u8]) -> bool {
        data.starts_with(b"P6")
    }

    fn load_image(&self, data: &[u8]) -> Option<Image> {
        if !self.is_supported(data) {
            return None;
        }

        let mut data = data.iter().skip(3).copied();
        let width = read_number(&mut data);
        let height = read_number(&mut data);
        let _ = read_number(&mut data);

        Some(Image::new(
            Dimension::new(width, height),
            ImageFormat::RGB(data.collect()),
        ))
    }
}

pub struct PBMLoader;

impl ImageLoader for PBMLoader {
    fn is_supported(&self, data: &[u8]) -> bool {
        data.starts_with(b"P4")
    }
    fn load_image(&self, data: &[u8]) -> Option<Image> {
        if !self.is_supported(data) {
            return None;
        }
        let mut data = data.iter().skip(3).copied();
        let width = read_number(&mut data);
        let height = read_number(&mut data);
        Some(Image::new(
            Dimension::new(width, height),
            ImageFormat::Bitmap {
                data: data.collect(),
                color: Color::new(255, 255, 255),
            },
        ))
    }
}

fn read_number(data: &mut impl Iterator<Item = u8>) -> u32 {
    let mut number = 0;

    while let Some(byte) = data.next() {
        match byte {
            b'0'..=b'9' => (),
            _ => break,
        }
        number = number * 10 + (byte - b'0') as u32;
    }

    number
}
