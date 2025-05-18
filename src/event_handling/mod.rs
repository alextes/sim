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
    pub middle_mouse_dragging: bool,
    pub last_mouse_pos: Option<(i32, i32)>,
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
        // lock state briefly to check if Esc should quit or go back
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
            if should_quit {
                return Signal::Quit;
            }
            continue; // skip further processing for this Escape event
        }

        // --- state-specific handling ---
        let mut state_guard = game_state.lock().unwrap();
        // clone the state *inside the lock* to pass to helpers
        // (needed because helpers might change the state via the guard)
        let current_state_clone = state_guard.clone();

        match current_state_clone {
            GameState::Playing => {
                // pass the guard itself to modify state directly if needed
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
            // delegate all build menu states to the build_menu handler
            GameState::BuildMenuSelectingSlotType
            | GameState::BuildMenuSelectingBuilding { .. }
            | GameState::BuildMenuError { .. } => {
                build_menu::handle_build_menu_input(
                    &event,
                    &current_state_clone, // pass immutable clone for logic read
                    world,
                    controls.entity_focus_index, // pass usize value (Copy)
                    &mut state_guard,            // pass mutable guard to allow state changes
                );
            }
        }
        // state_guard is dropped here automatically when going out of scope
    }
    Signal::Continue
}
