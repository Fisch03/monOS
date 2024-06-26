mod pbm;

trait ImageLoader {
    fn is_supported(&self, data: &[u8]) -> bool;
    fn load_image(&self, data: &[u8]) -> Image;
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_pbm(data: &[u8]) -> Self {
        pbm::PBMLoader.load_image(data)
    }

    pub fn detect_format(data: &[u8]) -> Option<&'static dyn ImageLoader> {
        if pbm::PBMLoader.is_supported(data) {
            Some(&pbm::PBMLoader)
        } else {
            None
        }
    }
}
