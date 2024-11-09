use crate::{Color, Dimension, Position};
pub use monos_std::io::SliceReader;

mod netpbm;

trait ImageLoader {
    fn load_image<T: Read>(&self, data: &T) -> Option<Image>;
}

#[derive(Clone)]
pub enum ImageFormat {
    RGB {
        data: Vec<u8>,
        alpha_val: Option<Color>,
    }, // 3 bytes per pixel (r, g, b), optionally treat a certain color as transparent
    Bitmap {
        data: Vec<u8>,
        color: Color,
    }, // 1 bit per pixel (0 = transparent, 1 = opaque)
}

#[derive(Clone)]
pub struct Image {
    pub dimensions: Dimension,
    pub data: ImageFormat,
}

impl Image {
    pub const fn new(dimensions: Dimension, data: ImageFormat) -> Self {
        Self { dimensions, data }
    }

    pub fn get_pixel(&self, pos: Position) -> Color {
        match &self.data {
            ImageFormat::RGB { data, .. } => {
                let offset = (pos.y * self.dimensions.width as i64 + pos.x) as usize * 3;
                Color::new(data[offset], data[offset + 1], data[offset + 2])
            }
            ImageFormat::Bitmap { data, color } => {
                let bytes_per_row = self.dimensions.width as usize / 8
                    + if self.dimensions.width % 8 != 0 { 1 } else { 0 };
                let byte_offset = (pos.y * bytes_per_row as i64 + pos.x / 8) as usize;
                let bit_offset = pos.x % 8;
                let bit_offset = 7 - bit_offset;
                if data[byte_offset] & (1 << bit_offset) != 0 {
                    *color
                } else {
                    Color::new(0, 0, 0)
                }
            }
        }
    }

    #[inline]
    pub const fn dimensions(&self) -> Dimension {
        self.dimensions
    }

    #[inline(always)]
    pub fn from_ppm<T: Read>(data: &T) -> Option<Self> {
        netpbm::PPMLoader.load_image(data)
    }

    pub fn from_pbm<T: Read>(data: &T) -> Option<Self> {
        netpbm::PBMLoader.load_image(data)
    }

    pub fn detect_format<T: Read + Seek>(data: T) -> Option<Image> {
        data.set_pos(0);
        if let Some(image) = Image::from_ppm(&data) {
            return Some(image);
        }

        data.set_pos(0);
        if let Some(image) = Image::from_pbm(&data) {
            return Some(image);
        }
        None
    }

    pub fn set_transparent_color(&mut self, color: Color) {
        match &mut self.data {
            ImageFormat::RGB { alpha_val, .. } => {
                *alpha_val = Some(color);
            }
            _ => (),
        }
    }

    pub fn set_opaque_color(&mut self, color: Color) {
        match &mut self.data {
            ImageFormat::Bitmap { color: c, .. } => {
                *c = color;
            }
            _ => (),
        }
    }
}

impl core::fmt::Debug for Image {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Image")
            .field("dimensions", &self.dimensions)
            .finish()
    }
}
