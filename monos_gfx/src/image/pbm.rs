use super::{Image, ImageLoader};

pub struct PBMLoader;

impl ImageLoader for PBMLoader {
    fn is_supported(&self, data: &[u8]) -> bool {
        data.starts_with(b"P4")
    }

    fn load_image(&self, data: &[u8]) -> Image {
        assert!(self.is_supported(data));

        let mut data = data.iter().skip(3).copied();
        let width = read_number(&mut data);
        let height = read_number(&mut data);

        Image::new(width, height, data.collect())
    }
}

fn read_number(data: &mut impl Iterator<Item = u8>) -> u32 {
    let mut number = 0;

    while let Some(byte) = data.next() {
        if byte == b' ' {
            break;
        }
        number = number * 10 + (byte - b'0') as u32;
    }

    number
}
