use std::sync::MutexGuard;

use super::ControlState;
use crate::buildings::BuildingType;
use crate::input; // Import the new input module
use crate::render::Viewport;
use crate::world::{EntityId, World};
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

fn cycle_entity_focus(world: &World, controls: &mut ControlState, is_shift_pressed: bool) {
    if world.entities.is_empty() {
        controls.selection.clear();
        return;
    }

    let num_entities = world.entities.len();
    let current_index = controls
        .selection
        .first()
        .and_then(|id| world.entities.iter().position(|e| e == id));

    let next_index = if is_shift_pressed {
        // previous entity
        match current_index {
            Some(current) => {
                if current == 0 {
                    num_entities - 1
                } else {
                    current - 1
                }
            }
            None => num_entities - 1, // start from the end
        }
    } else {
        // next entity
        match current_index {
            Some(current) => (current + 1) % num_entities,
            None => 0, // start from the beginning
        }
    };
    controls.selection = vec![world.entities[next_index]];
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
        Keycode::F => {
            if !controls.selection.is_empty() {
                controls.track_mode = !controls.track_mode
            }
        }
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
            if controls.selection.len() == 1 {
                let selected_id = controls.selection[0];
                if world.is_player_controlled(selected_id)
                    && world.buildings.contains_key(&selected_id)
                {
                    **game_state_guard = GameState::BuildMenu;
                }
            }
        }
        Keycode::S => {
            if controls.selection.len() == 1 {
                let selected_id = controls.selection[0];
                if world.is_player_controlled(selected_id) {
                    if let Some(buildings) = world.buildings.get(&selected_id) {
                        let has_shipyard = buildings.slots.contains(&Some(BuildingType::Shipyard));
                        if has_shipyard {
                            **game_state_guard = GameState::ShipyardMenu;
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
                match input::get_entity_id_at_screen_coords(x, y, location_viewport, world) {
                    Some(id) => {
                        controls.selection = vec![id];
                    }
                    None => {
                        // start selection box
                        controls.selection_box_start = Some((x, y));
                        controls.last_mouse_pos = Some((x, y));
                        controls.selection.clear();
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
            let dest = location_viewport.screen_to_world_coords(x, y);
            for &id in &controls.selection {
                if world.ships.contains_key(&id) {
                    world.add_command(crate::command::Command::MoveShip {
                        ship_id: id,
                        destination: dest,
                    });
                }
            }
        }
        _ => {}
    }
}

fn handle_mouse_button_up(
    mouse_btn: &MouseButton,
    x: i32,
    y: i32,
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
) {
    if mouse_btn == &MouseButton::Middle {
        controls.middle_mouse_dragging = false;
        controls.last_mouse_pos = None;
    }
    if mouse_btn == &MouseButton::Left {
        if let Some(start_pos) = controls.selection_box_start {
            // finalize selection box
            let x1 = start_pos.0;
            let y1 = start_pos.1;
            let x2 = x;
            let y2 = y;

            let start_x = x1.min(x2);
            let start_y = y1.min(y2);
            let width = (x1 - x2).abs() as u32;
            let height = (y1 - y2).abs() as u32;

            let rect = sdl2::rect::Rect::new(start_x, start_y, width, height);

            let entities_in_box =
                input::get_entities_in_screen_rect(rect, location_viewport, world);

            // apply selection logic
            apply_box_selection_logic(controls, world, &entities_in_box);

            controls.selection_box_start = None;
            controls.last_mouse_pos = None;
        }

        controls.ctrl_left_mouse_dragging = false;
        if controls.last_mouse_pos.is_some() {
            controls.last_mouse_pos = None;
        }
    }
}

fn apply_box_selection_logic(
    controls: &mut ControlState,
    world: &World,
    entities_in_box: &[EntityId],
) {
    controls.selection.clear();

    let ships: Vec<EntityId> = entities_in_box
        .iter()
        .filter(|id| world.ships.contains_key(id))
        .cloned()
        .collect();

    if !ships.is_empty() {
        controls.selection = ships;
        return;
    }

    let planets: Vec<EntityId> = entities_in_box
        .iter()
        .filter(|id| world.get_render_glyph(**id) == 'p')
        .cloned()
        .collect();
    if planets.len() == 1 {
        controls.selection = planets;
        return;
    }

    let stars: Vec<EntityId> = entities_in_box
        .iter()
        .filter(|id| world.get_render_glyph(**id) == '*')
        .cloned()
        .collect();
    if stars.len() == 1 {
        controls.selection = stars;
        return;
    }

    let moons: Vec<EntityId> = entities_in_box
        .iter()
        .filter(|id| world.get_render_glyph(**id) == 'm')
        .cloned()
        .collect();
    if moons.len() == 1 {
        controls.selection = moons;
        return;
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
    } else if controls.selection_box_start.is_some() {
        // update drag selection endpoint
        controls.last_mouse_pos = Some((x, y));
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
        Event::MouseButtonUp {
            mouse_btn, x, y, ..
        } => {
            handle_mouse_button_up(mouse_btn, *x, *y, location_viewport, world, controls);
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
