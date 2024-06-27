use crate::Dimension;

mod pbm;

trait ImageLoader {
    fn is_supported(&self, data: &[u8]) -> bool;
    fn load_image(&self, data: &[u8]) -> Option<Image>;
}

pub struct Image {
    dimensions: Dimension,
    pub data: Vec<u8>,
}

impl Image {
    pub const fn new(dimensions: Dimension, data: Vec<u8>) -> Self {
        Self { dimensions, data }
    }

    #[inline]
    pub fn dimensions(&self) -> Dimension {
        self.dimensions
    }

    pub fn from_ppm(data: &[u8]) -> Option<Self> {
        pbm::PPMLoader.load_image(data)
    }

    pub fn detect_format(data: &[u8]) -> Option<Image> {
        if let Some(image) = Image::from_ppm(data) {
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
