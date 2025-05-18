use crate::input; // Import the new input module
use crate::render::Viewport;
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
    // return Signal only if quitting
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
                *entity_focus_index = (*entity_focus_index + 1) % world.entities.len();
            }
        }
        Event::KeyDown {
            keycode: Some(Keycode::B),
            ..
        } => {
            // check if currently selected entity can have buildings
            if !world.entities.is_empty() {
                let selected_id = world.entities[*entity_focus_index];
                if world.buildings.contains_key(&selected_id) {
                    **game_state_guard = GameState::BuildMenuSelectingSlotType;
                } else {
                    // optionally provide feedback
                    // **game_state_guard = GameState::BuildMenuError { message: "Cannot build on this entity".to_string() };
                }
            }
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
        Event::MouseButtonDown { x, y, .. } => {
            if let Some(idx) =
                input::get_entity_index_at_screen_coords(*x, *y, location_viewport, world)
            {
                *entity_focus_index = idx;
            }
        }
        _ => {} // ignore other events in Playing state
    }
    None // no quit signal
}
