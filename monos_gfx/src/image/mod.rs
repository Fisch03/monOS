use crate::{Color, Dimension};

mod netpbm;

trait ImageLoader {
    fn load_image<T: Read>(&self, data: &T) -> Option<Image>;
}

pub enum ImageFormat {
    RGB(Vec<u8>),                           // 3 bytes per pixel (r, g, b)
    Bitmap { data: Vec<u8>, color: Color }, // 1 bit per pixel (0 = transparent, 1 = opaque)
}

pub struct Image {
    dimensions: Dimension,
    pub data: ImageFormat,
}

impl Image {
    pub const fn new(dimensions: Dimension, data: ImageFormat) -> Self {
        Self { dimensions, data }
    }

    #[inline]
    pub fn dimensions(&self) -> Dimension {
        self.dimensions
    }

    pub fn from_ppm<T: Read>(data: &T) -> Option<Self> {
        netpbm::PPMLoader.load_image(data)
    }

    pub fn from_pbm<T: Read>(data: &T) -> Option<Self> {
        netpbm::PBMLoader.load_image(data)
    }

    pub fn detect_format<T: Read + Seek>(data: T) -> Option<Image> {
        data.seek(0);
        if let Some(image) = Image::from_ppm(&data) {
            return Some(image);
        }

        data.seek(0);
        if let Some(image) = Image::from_pbm(&data) {
            return Some(image);
        }
        None
    }
}

impl core::fmt::Debug for Image {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Image")
            .field("dimensions", &self.dimensions)
            .finish()
    }
}
