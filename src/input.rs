//! winit input handling and screen->world entity picking.
//!
//! replaces the old sdl `event_handling` + `input` modules. stage 2 wires the
//! camera (pan/zoom) and single-click selection so the rendered world is
//! navigable. the menu-opening keys (b/s/r), box-select, and overlays return in
//! stage 3 alongside the egui ui.

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::app::GameState;
use crate::command::Command;
use crate::control_state::ControlState;
use crate::location::PointF64;
use crate::viewport::Viewport;
use crate::world::{EntityId, World};

/// world distance panned per arrow-key press at zoom 1.0.
const KEY_PAN_AT_ZOOM_1: f64 = 0.25;
/// mouse-wheel zoom step.
const WHEEL_ZOOM_FACTOR: f64 = 1.2;

/// handle one winit window event that egui did not consume.
pub fn handle_window_event(
    event: &WindowEvent,
    viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state: &mut GameState,
) {
    // cursor position and modifier state are tracked regardless of game state.
    match event {
        WindowEvent::ModifiersChanged(modifiers) => {
            let state = modifiers.state();
            controls.ctrl_down = state.control_key() || state.super_key();
            controls.shift_down = state.shift_key();
            return;
        }
        WindowEvent::CursorMoved { position, .. } => {
            handle_cursor_moved(*position, viewport, controls);
            return;
        }
        _ => {}
    }

    // the rest are gameplay bindings, only active while playing.
    if *game_state != GameState::Playing {
        return;
    }

    match event {
        WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
            handle_keydown(event, viewport, world, controls);
        }
        WindowEvent::MouseInput { state, button, .. } => {
            handle_mouse_button(*state, *button, viewport, world, controls);
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let scroll = match delta {
                MouseScrollDelta::LineDelta(_, y) => *y as f64,
                MouseScrollDelta::PixelDelta(p) => p.y,
            };
            if let Some(pos) = controls.last_mouse_pos {
                if scroll > 0.0 {
                    viewport.zoom_at(WHEEL_ZOOM_FACTOR, pos);
                } else if scroll < 0.0 {
                    viewport.zoom_at(1.0 / WHEEL_ZOOM_FACTOR, pos);
                }
            }
        }
        _ => {}
    }
}

fn handle_cursor_moved(
    position: PhysicalPosition<f64>,
    viewport: &mut Viewport,
    controls: &mut ControlState,
) {
    let new = (position.x as i32, position.y as i32);
    let prev = controls.last_mouse_pos;
    controls.last_mouse_pos = Some(new);

    // middle-drag or ctrl+left-drag pans the camera.
    if controls.middle_mouse_dragging || controls.ctrl_left_mouse_dragging {
        if let Some((px, py)) = prev {
            let scale = viewport.world_tile_pixel_size_on_screen();
            viewport.anchor.x -= (new.0 - px) as f64 / scale;
            viewport.anchor.y -= (new.1 - py) as f64 / scale;
        }
    }
}

fn handle_keydown(
    event: &KeyEvent,
    viewport: &mut Viewport,
    world: &World,
    controls: &mut ControlState,
) {
    let PhysicalKey::Code(code) = event.physical_key else {
        return;
    };
    let pan = KEY_PAN_AT_ZOOM_1 / viewport.zoom.max(0.01);

    match code {
        KeyCode::ArrowUp => viewport.anchor.y -= pan,
        KeyCode::ArrowDown => viewport.anchor.y += pan,
        KeyCode::ArrowLeft => viewport.anchor.x -= pan,
        KeyCode::ArrowRight => viewport.anchor.x += pan,
        KeyCode::Equal | KeyCode::NumpadAdd => viewport.zoom_in(),
        KeyCode::Minus | KeyCode::NumpadSubtract => viewport.zoom_out(),
        KeyCode::Tab if !event.repeat => cycle_entity_focus(world, controls, controls.shift_down),
        KeyCode::KeyF if !event.repeat => {
            if !controls.selection.is_empty() {
                controls.track_mode = !controls.track_mode;
            }
        }
        KeyCode::Space if !event.repeat => controls.paused = !controls.paused,
        KeyCode::Backquote if !event.repeat => {
            controls.sim_speed = match controls.sim_speed {
                1 => 2,
                2 => 3,
                _ => 1,
            };
        }
        _ => {}
    }
}

fn handle_mouse_button(
    state: ElementState,
    button: MouseButton,
    viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
) {
    let Some((x, y)) = controls.last_mouse_pos else {
        return;
    };

    match (state, button) {
        (ElementState::Pressed, MouseButton::Left) => {
            if controls.ctrl_down {
                controls.ctrl_left_mouse_dragging = true;
            } else {
                match get_entity_id_at_screen_coords(x, y, viewport, world) {
                    Some(id) => controls.selection = vec![id],
                    None => {
                        controls.selection.clear();
                        controls.track_mode = false;
                    }
                }
            }
        }
        (ElementState::Released, MouseButton::Left) => {
            controls.ctrl_left_mouse_dragging = false;
        }
        (ElementState::Pressed, MouseButton::Middle) => controls.middle_mouse_dragging = true,
        (ElementState::Released, MouseButton::Middle) => controls.middle_mouse_dragging = false,
        (ElementState::Pressed, MouseButton::Right) => {
            // move-order the selected ships to the clicked cell center.
            let precise = viewport.screen_to_world_coords(x, y);
            let destination = PointF64 {
                x: precise.x.floor() + 0.5,
                y: precise.y.floor() + 0.5,
            };
            let ships: Vec<EntityId> = controls
                .selection
                .iter()
                .copied()
                .filter(|id| world.ships.contains_key(id))
                .collect();
            for ship_id in ships {
                world.add_command(Command::MoveShip {
                    ship_id,
                    destination,
                });
            }
        }
        _ => {}
    }
}

/// cycle the single selection forward (or backward with shift) through entities.
fn cycle_entity_focus(world: &World, controls: &mut ControlState, reverse: bool) {
    if world.entities.is_empty() {
        controls.selection.clear();
        return;
    }
    let count = world.entities.len();
    let current = controls
        .selection
        .first()
        .and_then(|id| world.entities.iter().position(|e| e == id));

    let next = if reverse {
        match current {
            Some(0) => count - 1,
            Some(i) => i - 1,
            None => count - 1,
        }
    } else {
        match current {
            Some(i) => (i + 1) % count,
            None => 0,
        }
    };
    controls.selection = vec![world.entities[next]];
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
