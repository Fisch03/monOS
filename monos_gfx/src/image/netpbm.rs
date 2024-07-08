use crate::{Color, Dimension};

use super::{Image, ImageFormat, ImageLoader};

pub struct PPMLoader;

fn check_header<T: Read>(data: &T, expected: &[u8]) -> bool {
    let mut header = [0u8; 3];
    data.read(&mut header);
    header.starts_with(expected)
}

fn read_number(mut data: &[u8]) -> (u32, &[u8]) {
    let mut number = 0;

    while data.len() > 0 {
        let byte = data[0];

        match byte {
            b'0'..=b'9' => (),
            _ => break,
        }
        number = number * 10 + (byte - b'0') as u32;

        data = &data[1..];
    }

    (number, data)
}

fn parse_header<T: Read>(data: &T, expected_format: &[u8]) -> Option<(Dimension, Vec<u8>)> {
    if !check_header(data, expected_format) {
        return None;
    }

    let mut size_header = [0u8; 16]; // this WILL break if the header is longer than 16 bytes. i am
                                     // assuming it won't be because the image dimensions would need
                                     // to be way larger than anything usable by the OS anyway.

    data.read(&mut size_header);

    let (width, mut size_header) = read_number(&size_header);
    size_header = &size_header[1..]; // skip space
    let (height, mut size_header) = read_number(size_header);
    size_header = &size_header[1..]; // skip newline

    Some((Dimension::new(width, height), size_header.to_vec()))
}

impl ImageLoader for PPMLoader {
    fn load_image<T: Read>(&self, data: &T) -> Option<Image> {
        let (dimensions, header_remaining) = parse_header(data, b"P6")?;

        let (_, mut header_remaining) = read_number(&header_remaining); // we just assume the max color value is 255.
                                                                        // because who tf uses anything else.
        header_remaining = &header_remaining[1..]; // skip newline

        let mut pixel_data = Vec::from(header_remaining);
        let size_bytes = dimensions.width as usize * dimensions.height as usize * 3;

        let start_offset = pixel_data.len();
        pixel_data.resize(size_bytes, 0);
        data.read(&mut pixel_data[start_offset..]);

        Some(Image::new(
            dimensions,
            ImageFormat::RGB {
                data: pixel_data,
                alpha_val: Some(Color::new(0, 0, 0)),
            },
        ))
    }
}

pub struct PBMLoader;

impl ImageLoader for PBMLoader {
    fn load_image<T: Read>(&self, data: &T) -> Option<Image> {
        let (dimensions, header_remaining) = parse_header(data, b"P4")?;

        let mut pixel_data = Vec::from(header_remaining);
        let size_bytes = dimensions.width as usize * dimensions.height as usize / 8;
        let start_offset = pixel_data.len();

        pixel_data.reserve(size_bytes);
        unsafe { pixel_data.set_len(size_bytes) };
        data.read(&mut pixel_data[start_offset..]);

        Some(Image::new(
            dimensions,
            ImageFormat::Bitmap {
                data: pixel_data,
                color: Color::new(255, 255, 255),
            },
        ))
    }
}
