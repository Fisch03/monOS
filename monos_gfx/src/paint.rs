use crate::Image;
use spin::lazy::Lazy;

static PAINT_SPLASHES: Lazy<Vec<Image>> = Lazy::new(|| {
    let mut images = Vec::new();

    // images.push(Image::from_pbm(include_bytes!("../assets/splash_1.pbm")).unwrap());
    // images.push(Image::from_pbm(include_bytes!("../assets/splash_2.pbm")).unwrap());
    // images.push(Image::from_pbm(include_bytes!("../assets/splash_3.pbm")).unwrap());
    // images.push(Image::from_pbm(include_bytes!("../assets/splash_4.pbm")).unwrap());
    // images.push(Image::from_pbm(include_bytes!("../assets/splash_5.pbm")).unwrap());

    images
});
