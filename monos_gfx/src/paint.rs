// static PAINT_SPLASHES: Lazy<Vec<Image>> = Lazy::new(|| {
//     let images = Vec::new();
//
//     // images.push(Image::from_pbm(include_bytes!("../assets/splash_1.pbm")).unwrap());
//     // images.push(Image::from_pbm(include_bytes!("../assets/splash_2.pbm")).unwrap());
//     // images.push(Image::from_pbm(include_bytes!("../assets/splash_3.pbm")).unwrap());
//     // images.push(Image::from_pbm(include_bytes!("../assets/splash_4.pbm")).unwrap());
//     // images.push(Image::from_pbm(include_bytes!("../assets/splash_5.pbm")).unwrap());
//
//     images
// });

use crate::{types::*, Framebuffer, Image};
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

pub struct PaintFramebuffer<'fb> {
    fb: Framebuffer<'fb>,
    splashes: Vec<Image>,
    seed: SplatSeed,
}

impl<'fb> PaintFramebuffer<'fb> {
    pub fn new(fb: Framebuffer<'fb>) -> Self {
        let splashes = syscall::list("data/splashes");
        let splashes = splashes
            .into_iter()
            .filter_map(|entry| File::open(&dbg!(entry)))
            .filter_map(|file| Image::from_pbm(&dbg!(file)))
            .collect();

        Self {
            fb,
            seed: SplatSeed(100),
            splashes,
        }
    }

    pub fn splat(&mut self, position: Position, color: Color) -> SplatSeed {
        let seed = self.seed.next();
        self.splat_seeded(position, color, seed);
        seed
    }

    pub fn splat_seeded(&mut self, position: Position, color: Color, seed: SplatSeed) {
        let mut rng = SmallRng::seed_from_u64(seed.0);
        let splash = self.splashes.choose_mut(&mut rng).unwrap();
        splash.set_opaque_color(color);
        self.fb.draw_img(splash, position);
    }
}

impl<'fb> core::ops::Deref for PaintFramebuffer<'fb> {
    type Target = Framebuffer<'fb>;

    fn deref(&self) -> &Self::Target {
        &self.fb
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SplatSeed(u64);

impl SplatSeed {
    fn next(&mut self) -> SplatSeed {
        let seed = self.0;
        self.0 = seed.wrapping_add(1);
        SplatSeed(seed)
    }
}
