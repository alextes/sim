//! viewport: world<->screen coordinate transforms and camera state.
//!
//! ported from the old sdl `render::viewport` module. this is pure f64 math
//! with no rendering dependency; the gpu sprite batch (stage 2) consumes it.

use crate::location::{Point, PointF64};

/// size of a single world tile in screen pixels at zoom 1.0.
pub const TILE_PIXEL_WIDTH: u32 = 9;

/// initial window size (also the default viewport size).
pub const INITIAL_WINDOW_WIDTH: u32 = 800;
pub const INITIAL_WINDOW_HEIGHT: u32 = 600;

pub struct Viewport {
    /// which universe coordinate the center of the viewport is looking at.
    pub anchor: PointF64,
    /// magnification. zoom > 1.0 means world tiles appear larger.
    pub zoom: f64,
    /// width of the viewport's rendering area on screen, in pixels.
    pub screen_pixel_width: u32,
    /// height of the viewport's rendering area on screen, in pixels.
    pub screen_pixel_height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            anchor: PointF64 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            screen_pixel_width: INITIAL_WINDOW_WIDTH,
            screen_pixel_height: INITIAL_WINDOW_HEIGHT,
        }
    }
}

impl Viewport {
    pub fn world_tile_pixel_size_on_screen(&self) -> f64 {
        (TILE_PIXEL_WIDTH as f64 * self.zoom).max(0.001)
    }

    pub fn screen_to_world_coords(&self, screen_x: i32, screen_y: i32) -> PointF64 {
        let world_tile_actual_pixel_size_on_screen =
            (TILE_PIXEL_WIDTH as f64 * self.zoom).max(0.001);

        let view_world_origin_x = self.anchor.x
            - (self.screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
        let view_world_origin_y = self.anchor.y
            - (self.screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

        let world_x =
            view_world_origin_x + (screen_x as f64 / world_tile_actual_pixel_size_on_screen);
        let world_y =
            view_world_origin_y + (screen_y as f64 / world_tile_actual_pixel_size_on_screen);

        PointF64 {
            x: world_x,
            y: world_y,
        }
    }

    // used by box-select / overlays (overlay follow-up).
    #[allow(dead_code)]
    pub fn world_to_screen_coords(&self, world_pos: Point) -> (i32, i32) {
        let world_tile_actual_pixel_size_on_screen = self.world_tile_pixel_size_on_screen();

        let view_world_origin_x = self.anchor.x
            - (self.screen_pixel_width as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;
        let view_world_origin_y = self.anchor.y
            - (self.screen_pixel_height as f64 / 2.0) / world_tile_actual_pixel_size_on_screen;

        let screen_x =
            (world_pos.x as f64 - view_world_origin_x) * world_tile_actual_pixel_size_on_screen;
        let screen_y =
            (world_pos.y as f64 - view_world_origin_y) * world_tile_actual_pixel_size_on_screen;

        (screen_x.round() as i32, screen_y.round() as i32)
    }

    pub fn center_on_entity(&mut self, x: i32, y: i32) {
        self.anchor.x = x as f64;
        self.anchor.y = y as f64;
    }

    pub fn zoom_in(&mut self) {
        self.zoom *= 1.2;
        self.zoom = self.zoom.clamp(0.05, 10.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom /= 1.2;
        self.zoom = self.zoom.clamp(0.05, 10.0);
    }

    pub fn zoom_at(&mut self, zoom_factor: f64, mouse_screen_pos: (i32, i32)) {
        let world_pos_before_zoom =
            self.screen_to_world_coords(mouse_screen_pos.0, mouse_screen_pos.1);

        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(0.05, 10.0);

        let new_world_tile_pixel_size = (TILE_PIXEL_WIDTH as f64 * self.zoom).max(0.001);
        let mouse_offset_from_center_x =
            mouse_screen_pos.0 as f64 - self.screen_pixel_width as f64 / 2.0;
        let mouse_offset_from_center_y =
            mouse_screen_pos.1 as f64 - self.screen_pixel_height as f64 / 2.0;

        self.anchor.x =
            world_pos_before_zoom.x - mouse_offset_from_center_x / new_world_tile_pixel_size;
        self.anchor.y =
            world_pos_before_zoom.y - mouse_offset_from_center_y / new_world_tile_pixel_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.anchor, PointF64 { x: 0.0, y: 0.0 });
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.screen_pixel_width, INITIAL_WINDOW_WIDTH);
        assert_eq!(vp.screen_pixel_height, INITIAL_WINDOW_HEIGHT);
    }

    #[test]
    fn test_center_on_entity() {
        let mut vp = Viewport::default();
        vp.center_on_entity(10, 20);
        assert_eq!(vp.anchor, PointF64 { x: 10.0, y: 20.0 });
    }

    #[test]
    fn test_zoom_in_out() {
        let mut vp = Viewport::default();
        let original_zoom = vp.zoom;
        vp.zoom_in();
        assert!(vp.zoom > original_zoom);
        vp.zoom_out();
        let diff = (vp.zoom - original_zoom).abs();
        assert!(diff < f64::EPSILON);
    }

    #[test]
    fn test_screen_to_world_coords() {
        let mut vp = Viewport {
            anchor: PointF64 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            screen_pixel_width: 800,
            screen_pixel_height: 600,
        };

        // case 1: no zoom, anchor at origin
        vp.zoom = 1.0;
        vp.anchor = PointF64 { x: 0.0, y: 0.0 };
        let coords = vp.screen_to_world_coords(400, 300); // screen center
        assert!((coords.x - 0.0).abs() < 1e-9);
        assert!((coords.y - 0.0).abs() < 1e-9);

        // case 2: zoomed in, anchor at origin
        vp.zoom = 2.0;
        let coords = vp.screen_to_world_coords(400, 300); // screen center
        assert!((coords.x - 0.0).abs() < 1e-9);
        assert!((coords.y - 0.0).abs() < 1e-9);
        // top-left screen should be top-left of smaller world view
        let tile_size = TILE_PIXEL_WIDTH as f64;
        let expected_x = 0.0 - (800.0 / 2.0) / (tile_size * 2.0);
        let expected_y = 0.0 - (600.0 / 2.0) / (tile_size * 2.0);
        let coords_tl = vp.screen_to_world_coords(0, 0);
        assert!((coords_tl.x - expected_x).abs() < 1e-9);
        assert!((coords_tl.y - expected_y).abs() < 1e-9);

        // case 3: zoomed out, anchor offset
        vp.zoom = 0.5;
        vp.anchor = PointF64 { x: 100.0, y: -50.0 };
        let coords_center = vp.screen_to_world_coords(400, 300); // screen center
        assert!((coords_center.x - 100.0).abs() < 1e-9);
        assert!((coords_center.y - -50.0).abs() < 1e-9);
    }
}
