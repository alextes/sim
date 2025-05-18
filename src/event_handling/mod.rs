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

#[derive(Debug)]
pub struct ControlState {
    pub entity_focus_index: usize,
    pub debug_enabled: bool,
    pub track_mode: bool,
    pub sim_speed: u32,
    pub paused: bool,
}

pub fn handle_events(
    event_pump: &mut EventPump,
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state: Arc<Mutex<GameState>>,
) -> Signal {
    for event in event_pump.poll_iter() {
        // --- global escape handling ---
        if let Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } = event
        {
            let mut state_guard = game_state.lock().unwrap();
            match *state_guard {
                GameState::Playing => {
                    *state_guard = GameState::PauseMenu; // Open PauseMenu instead of quitting
                    // false // should_quit is no longer used here directly
                }
                GameState::PauseMenu => {
                    *state_guard = GameState::Playing; // Close PauseMenu
                    // false
                }
                GameState::BuildMenuSelectingSlotType => {
                    *state_guard = GameState::Playing; // Close build menu
                    // false
                }
                GameState::BuildMenuSelectingBuilding { .. } => {
                    // Go back to selecting slot type, or could go to Playing
                    *state_guard = GameState::BuildMenuSelectingSlotType;
                    // false
                }
                GameState::BuildMenuError { .. } => {
                    *state_guard = GameState::Playing; // Close error message
                    // false
                }
            }
            // if should_quit { // This logic is removed as Esc no longer quits directly
            //     return Signal::Quit;
            // }
            continue; // skip further processing for this Escape event
        }

        // --- state-specific handling ---
        let mut state_guard = game_state.lock().unwrap();
        let current_state_clone = state_guard.clone();

        match current_state_clone {
            GameState::Playing => {
                if let Some(signal) = playing::handle_playing_input(
                    &event,
                    location_viewport,
                    world,
                    controls,
                    &mut state_guard,
                ) {
                    return signal;
                }
            }
            GameState::PauseMenu => {
                // Handle PauseMenu specific inputs (e.g., 'Q' to quit)
                if let Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } = event
                {
                    return Signal::Quit;
                }
                // Esc is handled by the global escape handler above
            }
            GameState::BuildMenuSelectingSlotType
            | GameState::BuildMenuSelectingBuilding { .. }
            | GameState::BuildMenuError { .. } => {
                build_menu::handle_build_menu_input(
                    &event,
                    &current_state_clone,
                    world,
                    controls.entity_focus_index,
                    &mut state_guard,
                );
            }
        }
    }
    Signal::Continue
}
