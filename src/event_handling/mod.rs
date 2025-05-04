use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::sync::{Arc, Mutex};

use crate::render::Viewport;
use crate::world::World;
use crate::GameState;

mod build_menu;
mod playing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Quit,
    Continue,
}

pub fn handle_events(
    event_pump: &mut EventPump,
    location_viewport: &mut Viewport,
    world: &mut World,
    entity_focus_index: &mut usize,
    debug_enabled: &mut bool,
    track_mode: &mut bool,
    game_state: Arc<Mutex<GameState>>,
) -> Signal {
    for event in event_pump.poll_iter() {
        // --- Global Escape Handling ---
        // Lock state briefly to check if Esc should quit or go back
        if let Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } = event
        {
            let mut state_guard = game_state.lock().unwrap();
            let should_quit = match *state_guard {
                GameState::Playing => true,
                GameState::BuildMenuSelectingSlotType => {
                    *state_guard = GameState::Playing;
                    false
                }
                GameState::BuildMenuSelectingBuilding { .. } => {
                    *state_guard = GameState::BuildMenuSelectingSlotType;
                    false
                }
                GameState::BuildMenuError { .. } => {
                    *state_guard = GameState::Playing;
                    false
                }
            };
            drop(state_guard); // Drop lock before potential return
            if should_quit {
                return Signal::Quit;
            }
            continue; // Skip further processing for this Escape event
        }

        // --- State-Specific Handling ---
        let mut state_guard = game_state.lock().unwrap();
        // Clone the state *inside the lock* to pass to helpers
        // (needed because helpers might change the state via the guard)
        let current_state_clone = state_guard.clone();

        match current_state_clone {
            GameState::Playing => {
                // Pass the guard itself to modify state directly if needed
                if let Some(signal) = playing::handle_playing_input(
                    &event,
                    location_viewport,
                    world,
                    entity_focus_index,
                    debug_enabled,
                    track_mode,
                    &mut state_guard, // Pass mutable guard
                ) {
                    drop(state_guard); // Drop lock before return
                    return signal;
                }
            }
            // Delegate all build menu states to the build_menu handler
            GameState::BuildMenuSelectingSlotType
            | GameState::BuildMenuSelectingBuilding { .. }
            | GameState::BuildMenuError { .. } => {
                build_menu::handle_build_menu_input(
                    &event,
                    &current_state_clone, // Pass immutable clone for logic read
                    world,
                    *entity_focus_index, // Pass usize value (Copy)
                    &mut state_guard,    // Pass mutable guard to allow state changes
                );
            }
        }
        // state_guard is dropped here automatically when going out of scope
    }
    Signal::Continue
}
