use crate::colors;
use crate::location::PointF64;
use crate::render::{SpriteSheetRenderer, Viewport, TILE_PIXEL_WIDTH};
use rand::Rng;
use sdl2::render::Canvas;
use sdl2::video::Window;

const NUM_BG_STARS: usize = 2000;
const BG_STAR_SPREAD: f64 = 200.0; // The area over which background stars are scattered - reduced for density

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

    pub fn render(
        &self,
        canvas: &mut Canvas<Window>,
        renderer: &SpriteSheetRenderer,
        viewport: &Viewport,
    ) {
        const BASE_PARALLAX_SCROLL_FACTOR: f64 = 0.05; // Base scroll, independent of zoom
        const ZOOM_INFLUENCE_ON_PARALLAX: f64 = 0.05; // How much zoom adds to the effect

        // The effective parallax factor determines how much the background 'slips' against the foreground.
        // A smaller factor means a greater slip (stronger parallax).
        // The speed ratio of bg to fg is (BASE + ZOOM_INF * zoom) / zoom = BASE/zoom + ZOOM_INF.
        // As zoom increases, this ratio decreases, strengthening the effect.
        let effective_parallax_factor =
            BASE_PARALLAX_SCROLL_FACTOR + ZOOM_INFLUENCE_ON_PARALLAX * viewport.zoom;

        // Calculate the background's viewport based on the main viewport and parallax factor.
        let bg_anchor_x = viewport.anchor.x * effective_parallax_factor;
        let bg_anchor_y = viewport.anchor.y * effective_parallax_factor;

        // The background does not zoom. This enhances the parallax effect.
        // The size of a background tile on screen is fixed.
        let bg_tile_pixel_size = TILE_PIXEL_WIDTH as f64;

        let view_world_origin_x =
            bg_anchor_x - (viewport.screen_pixel_width as f64 / 2.0) / bg_tile_pixel_size;
        let view_world_origin_y =
            bg_anchor_y - (viewport.screen_pixel_height as f64 / 2.0) / bg_tile_pixel_size;

        let visible_world_width = viewport.screen_pixel_width as f64 / bg_tile_pixel_size;
        let visible_world_height = viewport.screen_pixel_height as f64 / bg_tile_pixel_size;

        let view_bbox_world_x_min = view_world_origin_x;
        let view_bbox_world_x_max = view_world_origin_x + visible_world_width;
        let view_bbox_world_y_min = view_world_origin_y;
        let view_bbox_world_y_max = view_world_origin_y + visible_world_height;

        let original_alpha = renderer.texture_ref().alpha_mod();
        let old_blend_mode = canvas.blend_mode();
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

        for star in &self.stars {
            // Basic culling
            if star.pos.x + 1.0 > view_bbox_world_x_min
                && star.pos.x < view_bbox_world_x_max
                && star.pos.y + 1.0 > view_bbox_world_y_min
                && star.pos.y < view_bbox_world_y_max
            {
                let src_rect = renderer.tileset.get_rect(star.glyph);

                let screen_x = (star.pos.x - view_world_origin_x) * bg_tile_pixel_size;
                let screen_y = (star.pos.y - view_world_origin_y) * bg_tile_pixel_size;

                let dst_rect = sdl2::rect::Rect::new(
                    screen_x.round() as i32,
                    screen_y.round() as i32,
                    bg_tile_pixel_size as u32,
                    bg_tile_pixel_size as u32,
                );

                // Use a fixed color for all background stars
                renderer.set_texture_color_mod(colors::LGRAY.r, colors::LGRAY.g, colors::LGRAY.b);
                renderer.texture.borrow_mut().set_alpha_mod(star.alpha);

                canvas
                    .copy(&renderer.texture_ref(), Some(src_rect), Some(dst_rect))
                    .unwrap();
            }
        }

        // Restore original alpha and blend mode
        renderer.texture.borrow_mut().set_alpha_mod(original_alpha);
        canvas.set_blend_mode(old_blend_mode);
    }
}
