use monos_gfx::{Color, Framebuffer, Position};

use micromath::F32Ext;
use rand::{rngs::SmallRng, Rng, SeedableRng};

const MAX_LIFETIME: u64 = 60;

pub struct PaintFramebuffer<'fb> {
    fb: Framebuffer<'fb>,
    splatters: Vec<PaintSplatter>,
    seed: SplatSeed,
}

impl<'fb> PaintFramebuffer<'fb> {
    pub fn new(fb: Framebuffer<'fb>) -> Self {
        Self {
            fb,
            splatters: Vec::new(),
            seed: SplatSeed(0),
        }
    }

    pub fn has_splatters(&self) -> bool {
        !self.splatters.is_empty()
    }

    pub fn splat(&mut self, position: Position, color: Color) -> SplatSeed {
        let seed = self.seed.next();
        self.splat_seeded(position, color, seed);
        seed
    }

    pub fn splat_seeded(&mut self, position: Position, color: Color, seed: SplatSeed) {
        self.splatters
            .push(PaintSplatter::new(position, color, seed));
    }

    pub fn draw(&mut self) {
        for splatter in self.splatters.iter_mut() {
            splatter.splat_frame(&mut self.fb);
        }

        for i in (0..self.splatters.len()).rev() {
            if self.splatters[i].lifetime == 0 {
                self.splatters.swap_remove(i);
            }
        }
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

#[derive(Debug, Clone)]
struct PaintSplatter {
    color: Color,
    position: Position,
    particles: Vec<PaintParticle>,
    lifetime: u64,
    time_passed: u64,
    rng: SmallRng,
}

#[derive(Debug, Clone, Copy)]
struct PaintParticle {
    position: Position,
    velocity: (f32, f32),
}

impl PaintSplatter {
    fn new(position: Position, color: Color, seed: SplatSeed) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed.0);

        let particles = (0..rng.gen_range(5..10))
            .map(|_| {
                let angle = rng.gen_range(0.0..core::f32::consts::PI * 2.0);
                let speed = rng.gen_range(0.5..2.0);
                let velocity = (angle.cos() * speed, angle.sin() * speed);

                PaintParticle { position, velocity }
            })
            .collect();

        Self {
            color,
            position,
            particles,
            lifetime: rng.gen_range(30..MAX_LIFETIME),
            rng,
            time_passed: 0,
        }
    }

    fn splat_frame(&mut self, fb: &mut Framebuffer) {
        for particle in self.particles.iter_mut() {
            let radius = 4.0 - (self.lifetime as f32 / MAX_LIFETIME as f32 * 4.0);
            fb.draw_disc(particle.position, radius as u32, self.color);

            particle.position +=
                Position::new(particle.velocity.0 as i64, particle.velocity.1 as i64);

            particle.velocity.1 += 0.02;
            particle.velocity.0 *= 0.98;
        }

        let radius = self.time_passed.min(10) as i64;
        let pos = self.position
            + Position::new(
                self.rng.gen_range(-radius..radius),
                self.rng.gen_range(-radius..radius),
            );
        fb.draw_disc(pos, radius as u32, self.color);

        self.lifetime -= 1;
        self.time_passed += 1;
    }
}
