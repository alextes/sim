//! parallax background starfield. ported from the old sdl `render::background`;
//! the data + parallax math stay, the draw loop becomes sprite-batch instance
//! emission (see `gfx::sprite_batch`).

use rand::Rng;

use crate::location::PointF64;

const NUM_BG_STARS: usize = 2000;
/// the area over which background stars are scattered.
const BG_STAR_SPREAD: f64 = 200.0;

#[derive(Debug, Clone, Copy)]
pub struct BackgroundStar {
    pub pos: PointF64,
    pub glyph: char,
    pub alpha: u8,
}

#[derive(Debug)]
pub struct BackgroundLayer {
    pub stars: Vec<BackgroundStar>,
}

impl BackgroundLayer {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut stars = Vec::with_capacity(NUM_BG_STARS);
        let star_chars = ['*', '.', ','];

        for _ in 0..NUM_BG_STARS {
            let x = rng.random_range(-BG_STAR_SPREAD..BG_STAR_SPREAD);
            let y = rng.random_range(-BG_STAR_SPREAD..BG_STAR_SPREAD);
            let glyph = star_chars[rng.random_range(0..star_chars.len())];
            let alpha = rng.random_range(20..40);

            stars.push(BackgroundStar {
                pos: PointF64 { x, y },
                glyph,
                alpha,
            });
        }

        Self { stars }
    }
}
