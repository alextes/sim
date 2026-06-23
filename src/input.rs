//! winit input handling and screen->world entity picking.
//!
//! replaces the old sdl `event_handling` + `input` modules. stage 1 only tracks
//! the cursor and logs events; the per-state game bindings come back in later
//! stages. the picking helpers are ported and ready for stage 2/3 wiring.

use tracing::debug;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;

use crate::app::GameState;
use crate::control_state::ControlState;
use crate::location::PointF64;
use crate::viewport::Viewport;
use crate::world::{EntityId, World};

/// handle one winit window event that egui did not consume.
///
/// stage 1: track the cursor position (winit has no current-position query) and
/// log input so we can confirm routing works. real game bindings return later.
pub fn handle_window_event(
    event: &WindowEvent,
    _viewport: &mut Viewport,
    _world: &mut World,
    controls: &mut ControlState,
    _game_state: &mut GameState,
) {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            controls.last_mouse_pos = Some((position.x as i32, position.y as i32));
        }
        WindowEvent::KeyboardInput { event, .. } => {
            if event.state == ElementState::Pressed {
                if let PhysicalKey::Code(code) = event.physical_key {
                    debug!(?code, "key pressed");
                }
            }
        }
        WindowEvent::MouseInput { state, button, .. } => {
            if *state == ElementState::Pressed {
                let pos = controls.last_mouse_pos;
                debug!(?button, ?pos, "mouse pressed");
            }
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let scroll = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y as f64,
                MouseScrollDelta::PixelDelta(p) => p.y,
            };
            debug!(scroll, "mouse wheel");
        }
        _ => {}
    }
}

/// a screen-space rectangle in physical pixels, replacing `sdl2::rect::Rect`.
#[derive(Debug, Clone, Copy)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl ScreenRect {
    /// build the axis-aligned rect spanning two corner points.
    pub fn from_corners(a: (i32, i32), b: (i32, i32)) -> Self {
        Self {
            x: a.0.min(b.0),
            y: a.1.min(b.1),
            w: (a.0 - b.0).unsigned_abs(),
            h: (a.1 - b.1).unsigned_abs(),
        }
    }

    /// true if the two rects overlap (touching edges do not count).
    pub fn intersects(&self, other: &ScreenRect) -> bool {
        let ax2 = self.x + self.w as i32;
        let ay2 = self.y + self.h as i32;
        let bx2 = other.x + other.w as i32;
        let by2 = other.y + other.h as i32;
        self.x < bx2 && ax2 > other.x && self.y < by2 && ay2 > other.y
    }
}

/// the entity id at the given screen coordinates, if any.
pub fn get_entity_id_at_screen_coords(
    screen_x: i32,
    screen_y: i32,
    viewport: &Viewport,
    world: &World,
) -> Option<EntityId> {
    let clicked_world_coords: PointF64 = viewport.screen_to_world_coords(screen_x, screen_y);

    let clicked_world_tile_x_i32 = clicked_world_coords.x.floor() as i32;
    let clicked_world_tile_y_i32 = clicked_world_coords.y.floor() as i32;

    world.iter_entities().find_map(|entity_id| {
        world.get_location(entity_id).and_then(|loc| {
            if loc.x == clicked_world_tile_x_i32 && loc.y == clicked_world_tile_y_i32 {
                Some(entity_id)
            } else {
                None
            }
        })
    })
}

/// all entities whose on-screen tile overlaps the given screen rectangle.
pub fn get_entities_in_screen_rect(
    rect: ScreenRect,
    viewport: &Viewport,
    world: &World,
) -> Vec<EntityId> {
    let mut entities = Vec::new();

    let world_tile_size_on_screen = viewport.world_tile_pixel_size_on_screen();

    for entity_id in world.iter_entities() {
        if let Some(pos) = world.get_location(entity_id) {
            let screen_coords = viewport.world_to_screen_coords(pos);

            let entity_rect = ScreenRect {
                x: screen_coords.0,
                y: screen_coords.1,
                w: world_tile_size_on_screen.round() as u32,
                h: world_tile_size_on_screen.round() as u32,
            };

            if rect.intersects(&entity_rect) {
                entities.push(entity_id);
            }
        }
    }
    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;
    use crate::viewport::Viewport;
    use crate::world::World;

    #[test]
    fn test_get_entity_at_screen_coords() {
        let mut world = World::default();
        let entity_pos = Point { x: 10, y: 20 };
        let entity_id = world.spawn_star("test_star".to_string(), entity_pos);

        let viewport = Viewport {
            anchor: PointF64 {
                x: entity_pos.x as f64,
                y: entity_pos.y as f64,
            },
            zoom: 1.0,
            screen_pixel_width: 800,
            screen_pixel_height: 600,
        };

        // click on the center of the screen, which should be where the entity is
        let result = get_entity_id_at_screen_coords(400, 300, &viewport, &world);
        assert_eq!(result, Some(entity_id));

        // click somewhere else
        let result_none = get_entity_id_at_screen_coords(0, 0, &viewport, &world);
        assert_eq!(result_none, None);
    }

    #[test]
    fn test_screen_rect_intersects() {
        let a = ScreenRect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
        };
        let overlapping = ScreenRect {
            x: 5,
            y: 5,
            w: 10,
            h: 10,
        };
        let disjoint = ScreenRect {
            x: 20,
            y: 20,
            w: 5,
            h: 5,
        };
        assert!(a.intersects(&overlapping));
        assert!(!a.intersects(&disjoint));
    }
}
