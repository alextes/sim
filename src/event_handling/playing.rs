use super::ControlState;
use crate::input; // Import the new input module
use crate::render::Viewport;
use crate::world::World;
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton; // Import MouseButton

// Import TILE_PIXEL_WIDTH (assuming it's pub in src/render/mod.rs or crate::render)
// If TILE_PIXEL_WIDTH is in src/render/mod.rs, and that file has `pub const TILE_PIXEL_WIDTH: i32 = 16;`
// then this import should work if src/render/mod.rs has `pub use super::TILE_PIXEL_WIDTH;` or similar
// Or, if it's directly in crate::render
use crate::render::TILE_PIXEL_WIDTH;

pub fn handle_playing_input(
    event: &Event,
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>,
) -> Option<super::Signal> {
    // return Signal only if quitting
    match event {
        Event::Quit { .. } => return Some(super::Signal::Quit),
        Event::KeyDown {
            keycode: Some(Keycode::F4),
            ..
        } => controls.debug_enabled = !controls.debug_enabled,
        Event::KeyDown {
            keycode: Some(Keycode::F),
            ..
        } => controls.track_mode = !controls.track_mode,
        Event::KeyDown {
            keycode: Some(Keycode::Up),
            ..
        } => location_viewport.anchor.y -= 0.25,
        Event::KeyDown {
            keycode: Some(Keycode::Down),
            ..
        } => location_viewport.anchor.y += 0.25,
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        } => location_viewport.anchor.x -= 0.25,
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        } => location_viewport.anchor.x += 0.25,
        Event::KeyDown {
            keycode: Some(Keycode::Tab),
            ..
        } => {
            if !world.entities.is_empty() {
                if controls.entity_focus_index >= world.entities.len() {
                    // If no valid entity is selected (e.g., usize::MAX or out of bounds after entity removal)
                    controls.entity_focus_index = 0; // Select the first entity
                } else {
                    // Cycle to the next entity
                    controls.entity_focus_index =
                        (controls.entity_focus_index + 1) % world.entities.len();
                }
            }
        }
        Event::KeyDown {
            keycode: Some(Keycode::B),
            ..
        } => {
            // check if currently selected entity can have buildings
            if !world.entities.is_empty() && controls.entity_focus_index < world.entities.len() {
                let selected_id = world.entities[controls.entity_focus_index];
                if world.buildings.contains_key(&selected_id) {
                    **game_state_guard = GameState::BuildMenuSelectingSlotType;
                } else {
                    // optionally provide feedback
                    // **game_state_guard = GameState::BuildMenuError { message: "Cannot build on this entity".to_string() };
                }
            }
        }
        // cycle simulation speed 1x -> 2x -> 3x -> 1x on backtick (`) key
        Event::KeyDown {
            keycode: Some(Keycode::Backquote),
            ..
        } => {
            controls.sim_speed = match controls.sim_speed {
                1 => 2,
                2 => 3,
                _ => 1,
            };
        }
        // toggle pause on Space key
        Event::KeyDown {
            keycode: Some(Keycode::Space),
            ..
        } => {
            controls.paused = !controls.paused;
        }
        // to use keypad plus
        Event::KeyDown {
            keycode: Some(Keycode::Plus),
            ..
        } => location_viewport.zoom_in(),
        // to use laptop plus
        Event::KeyDown {
            keycode: Some(Keycode::Equals),
            keymod,
            ..
        } if keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
            || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD) =>
        {
            location_viewport.zoom_in()
        }
        Event::KeyDown {
            keycode: Some(Keycode::Minus),
            ..
        } => location_viewport.zoom_out(),
        Event::MouseButtonDown { mouse_btn, x, y, .. } => {
            match mouse_btn {
                MouseButton::Left => {
                    match input::get_entity_index_at_screen_coords(*x, *y, location_viewport, world) {
                        Some(idx) => {
                            controls.entity_focus_index = idx;
                            // Note: Current behavior preserves track_mode on new selection.
                            // If track_mode should be reset or explicitly set, that logic would go here.
                        }
                        None => {
                            // Clicked on empty space, so deselect.
                            controls.entity_focus_index = usize::MAX; // Sentinel for "no selection"
                            controls.track_mode = false; // Turn off tracking mode
                        }
                    }
                }
                MouseButton::Middle => {
                    controls.middle_mouse_dragging = true;
                    controls.last_mouse_pos = Some((*x, *y));
                }
                _ => {} // Other buttons ignored for now
            }
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            if mouse_btn == &MouseButton::Middle {
                controls.middle_mouse_dragging = false;
                controls.last_mouse_pos = None;
            }
        }
        Event::MouseMotion { x, y, .. } => {
            if controls.middle_mouse_dragging {
                if let Some((last_x, last_y)) = controls.last_mouse_pos {
                    let delta_x = *x - last_x;
                    let delta_y = *y - last_y;

                    // Scale mouse delta to world coordinates
                    // This logic is similar to what's in src/render/viewport.rs and src/input/mod.rs
                    let world_tile_actual_pixel_size_on_screen =
                        (TILE_PIXEL_WIDTH as f64 * location_viewport.zoom).max(0.001);

                    let delta_x_world = delta_x as f64 / world_tile_actual_pixel_size_on_screen;
                    let delta_y_world = delta_y as f64 / world_tile_actual_pixel_size_on_screen;

                    location_viewport.anchor.x -= delta_x_world;
                    location_viewport.anchor.y -= delta_y_world;

                    controls.last_mouse_pos = Some((*x, *y));
                }
            }
        }
        _ => {} // ignore other events in Playing state
    }
    None // no quit signal
}
