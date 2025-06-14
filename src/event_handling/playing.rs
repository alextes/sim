use std::sync::MutexGuard;

use super::ControlState;
use crate::buildings::BuildingType;
use crate::input; // Import the new input module
use crate::render::Viewport;
use crate::world::World;
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

fn cycle_entity_focus(world: &World, controls: &mut ControlState, is_shift_pressed: bool) {
    if world.entities.is_empty() {
        return;
    }

    let num_entities = world.entities.len();

    if is_shift_pressed {
        // previous entity
        let next_index = match controls.entity_focus_index {
            Some(current) => {
                if current == 0 {
                    num_entities - 1
                } else {
                    current - 1
                }
            }
            None => num_entities - 1, // start from the end
        };
        controls.entity_focus_index = Some(next_index);
    } else {
        // next entity
        let next_index = match controls.entity_focus_index {
            Some(current) => (current + 1) % num_entities,
            None => 0, // start from the beginning
        };
        controls.entity_focus_index = Some(next_index);
    }
}

fn handle_keydown(
    keycode: Keycode,
    keymod: sdl2::keyboard::Mod,
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state_guard: &mut MutexGuard<'_, GameState>,
) {
    const KEY_PAN_WORLD_DISTANCE_AT_ZOOM_1: f64 = 0.25;
    let current_pan_amount = KEY_PAN_WORLD_DISTANCE_AT_ZOOM_1 / location_viewport.zoom.max(0.01);

    match keycode {
        Keycode::F4 => controls.debug_enabled = !controls.debug_enabled,
        Keycode::F => controls.track_mode = !controls.track_mode,
        Keycode::Up => location_viewport.anchor.y -= current_pan_amount,
        Keycode::Down => location_viewport.anchor.y += current_pan_amount,
        Keycode::Left => location_viewport.anchor.x -= current_pan_amount,
        Keycode::Right => location_viewport.anchor.x += current_pan_amount,
        Keycode::Tab => {
            let is_shift_pressed = keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
                || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD);
            cycle_entity_focus(world, controls, is_shift_pressed);
        }
        Keycode::B => {
            if let Some(index) = controls.entity_focus_index {
                if index < world.entities.len() {
                    let selected_id = world.entities[index];
                    if world.is_player_controlled(selected_id)
                        && world.buildings.contains_key(&selected_id)
                    {
                        **game_state_guard = GameState::BuildMenu;
                    }
                }
            }
        }
        Keycode::S => {
            if let Some(index) = controls.entity_focus_index {
                if index < world.entities.len() {
                    let selected_id = world.entities[index];
                    if world.is_player_controlled(selected_id) {
                        if let Some(buildings) = world.buildings.get(&selected_id) {
                            let has_shipyard =
                                buildings.slots.contains(&Some(BuildingType::Shipyard));
                            if has_shipyard {
                                **game_state_guard = GameState::ShipyardMenu;
                            }
                        }
                    }
                }
            }
        }
        Keycode::Backquote => {
            controls.sim_speed = match controls.sim_speed {
                1 => 2,
                2 => 3,
                _ => 1,
            };
        }
        Keycode::Space => {
            controls.paused = !controls.paused;
        }
        Keycode::Plus => location_viewport.zoom_in(),
        Keycode::Equals
            if keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD)
                || keymod.contains(sdl2::keyboard::Mod::RSHIFTMOD) =>
        {
            location_viewport.zoom_in()
        }
        Keycode::Minus => location_viewport.zoom_out(),
        _ => {}
    }
}

fn handle_mouse_button_down(
    mouse_btn: &MouseButton,
    x: i32,
    y: i32,
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
) {
    match mouse_btn {
        MouseButton::Left => {
            if controls.ctrl_down {
                controls.ctrl_left_mouse_dragging = true;
                controls.last_mouse_pos = Some((x, y));
            } else {
                match input::get_entity_index_at_screen_coords(x, y, location_viewport, world) {
                    Some(idx) => {
                        controls.entity_focus_index = Some(idx);
                    }
                    None => {
                        controls.entity_focus_index = None;
                        controls.track_mode = false;
                    }
                }
            }
        }
        MouseButton::Middle => {
            controls.middle_mouse_dragging = true;
            controls.last_mouse_pos = Some((x, y));
        }
        MouseButton::Right => {
            if let Some(index) = controls.entity_focus_index {
                if index < world.entities.len() {
                    let selected_id = world.entities[index];
                    if world.ships.contains_key(&selected_id) {
                        let dest = location_viewport.screen_to_world_coords(x, y);
                        world.add_command(crate::command::Command::MoveShip {
                            ship_id: selected_id,
                            destination: dest,
                        });
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_mouse_button_up(mouse_btn: &MouseButton, controls: &mut ControlState) {
    if mouse_btn == &MouseButton::Middle {
        controls.middle_mouse_dragging = false;
        controls.last_mouse_pos = None;
    }
    if mouse_btn == &MouseButton::Left {
        controls.ctrl_left_mouse_dragging = false;
        if controls.last_mouse_pos.is_some() {
            controls.last_mouse_pos = None;
        }
    }
}

fn handle_mouse_motion(
    x: i32,
    y: i32,
    location_viewport: &mut Viewport,
    controls: &mut ControlState,
) {
    if controls.middle_mouse_dragging || controls.ctrl_left_mouse_dragging {
        if let Some((last_x, last_y)) = controls.last_mouse_pos {
            let delta_x = x - last_x;
            let delta_y = y - last_y;

            let world_tile_actual_pixel_size_on_screen =
                location_viewport.world_tile_pixel_size_on_screen();

            let delta_x_world = delta_x as f64 / world_tile_actual_pixel_size_on_screen;
            let delta_y_world = delta_y as f64 / world_tile_actual_pixel_size_on_screen;

            location_viewport.anchor.x -= delta_x_world;
            location_viewport.anchor.y -= delta_y_world;

            controls.last_mouse_pos = Some((x, y));
        }
    }
}

fn handle_mouse_wheel(y: i32, mouse_pos: (i32, i32), location_viewport: &mut Viewport) {
    if y > 0 {
        location_viewport.zoom_at(1.2, mouse_pos);
    } else if y < 0 {
        location_viewport.zoom_at(1.0 / 1.2, mouse_pos);
    }
}

pub fn handle_playing_input(
    event: &Event,
    mouse_pos: (i32, i32),
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>,
) -> Option<super::Signal> {
    match event {
        Event::Quit { .. } => return Some(super::Signal::Quit),
        Event::KeyDown {
            keycode: Some(keycode),
            keymod,
            ..
        } => {
            handle_keydown(
                *keycode,
                *keymod,
                location_viewport,
                world,
                controls,
                game_state_guard,
            );
        }
        Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => {
            handle_mouse_button_down(mouse_btn, *x, *y, location_viewport, world, controls);
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            handle_mouse_button_up(mouse_btn, controls);
        }
        Event::MouseMotion { x, y, .. } => {
            handle_mouse_motion(*x, *y, location_viewport, controls);
        }
        Event::MouseWheel { y, .. } => {
            handle_mouse_wheel(*y, mouse_pos, location_viewport);
        }
        _ => {}
    }
    None
}
