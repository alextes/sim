use crate::render::{Viewport, TILE_PIXEL_WIDTH};
use crate::world::World;
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_playing_input(
    event: &Event,
    location_viewport: &mut Viewport,
    world: &mut World,
    entity_focus_index: &mut usize,
    debug_enabled: &mut bool,
    track_mode: &mut bool,
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>,
) -> Option<super::Signal> {
    // Return Signal only if quitting
    match event {
        Event::Quit { .. } => return Some(super::Signal::Quit),
        Event::KeyDown {
            keycode: Some(Keycode::F4),
            ..
        } => *debug_enabled = !*debug_enabled,
        Event::KeyDown {
            keycode: Some(Keycode::F),
            ..
        } => *track_mode = !*track_mode,
        Event::KeyDown {
            keycode: Some(Keycode::Up),
            ..
        } => location_viewport.anchor.y -= 1,
        Event::KeyDown {
            keycode: Some(Keycode::Down),
            ..
        } => location_viewport.anchor.y += 1,
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        } => location_viewport.anchor.x -= 1,
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        } => location_viewport.anchor.x += 1,
        Event::KeyDown {
            keycode: Some(Keycode::Tab),
            ..
        } => {
            if !world.entities.is_empty() {
                *entity_focus_index = (*entity_focus_index + 1) % world.entities.len();
                // Don't center viewport here if tracking, main loop handles it
                if !*track_mode {
                    let entity_id = world.entities[*entity_focus_index];
                    if let Some(loc) = world.get_location(entity_id) {
                        location_viewport.center_on_entity(loc.x, loc.y);
                    }
                }
            }
        }
        Event::KeyDown {
            keycode: Some(Keycode::B),
            ..
        } => {
            // Check if currently selected entity can have buildings
            if !world.entities.is_empty() {
                let selected_id = world.entities[*entity_focus_index];
                if world.buildings.contains_key(&selected_id) {
                    **game_state_guard = GameState::BuildMenuSelectingSlotType;
                } else {
                    // Optionally provide feedback
                    // **game_state_guard = GameState::BuildMenuError { message: "Cannot build on this entity".to_string() };
                }
            }
        }
        Event::KeyDown {
            keycode: Some(Keycode::Plus),
            ..
        } => location_viewport.zoom_in(),
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
        Event::MouseButtonDown { x, y, .. } => {
            let tile_pixel = (TILE_PIXEL_WIDTH as f64 * location_viewport.zoom) as i32;
            if tile_pixel > 0 {
                let tile_x = x / tile_pixel;
                let tile_y = y / tile_pixel;
                let half_w = location_viewport.width as i32 / 2;
                let half_h = location_viewport.height as i32 / 2;
                let world_x = location_viewport.anchor.x - half_w + tile_x;
                let world_y = location_viewport.anchor.y - half_h + tile_y;

                if let Some((idx, _)) = world.iter_entities().enumerate().find(|(_, id)| {
                    world
                        .get_location(*id)
                        .is_some_and(|loc| loc.x == world_x && loc.y == world_y)
                }) {
                    *entity_focus_index = idx;
                    // Don't center viewport here if tracking
                    if !*track_mode {
                        let entity_id = world.entities[idx];
                        if let Some(loc) = world.get_location(entity_id) {
                            location_viewport.center_on_entity(loc.x, loc.y);
                        }
                    }
                }
            }
        }
        _ => {} // Ignore other events in Playing state
    }
    None // No quit signal
}
