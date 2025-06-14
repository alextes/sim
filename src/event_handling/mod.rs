use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::sync::{Arc, Mutex};

use crate::render::Viewport;
use crate::world::{EntityId, World};
use crate::GameState;

mod build_menu;
mod main_menu;
mod playing;
mod shipyard_menu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Quit,
    Continue,
}

#[derive(Debug)]
pub struct ControlState {
    pub selection: Vec<EntityId>,
    pub debug_enabled: bool,
    pub track_mode: bool,
    pub sim_speed: u32,
    pub paused: bool,
    pub middle_mouse_dragging: bool,
    pub ctrl_left_mouse_dragging: bool,
    pub ctrl_down: bool,
    pub last_mouse_pos: Option<(i32, i32)>,
    pub selection_box_start: Option<(i32, i32)>,
}

pub fn handle_events(
    event_pump: &mut EventPump,
    location_viewport: &mut Viewport,
    world: &mut World,
    controls: &mut ControlState,
    game_state: Arc<Mutex<GameState>>,
) -> Signal {
    let mouse_pos = (event_pump.mouse_state().x(), event_pump.mouse_state().y());
    for event in event_pump.poll_iter() {
        // --- global key state tracking ---
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::LCtrl) | Some(Keycode::RCtrl),
                ..
            } => {
                controls.ctrl_down = true;
            }
            Event::KeyUp {
                keycode: Some(Keycode::LCtrl) | Some(Keycode::RCtrl),
                ..
            } => {
                controls.ctrl_down = false;
            }
            _ => {}
        }

        // handle window resize events directly, as they affect the viewport
        // and are not directly tied to game logic state.
        if let Event::Window {
            win_event: sdl2::event::WindowEvent::Resized(width, height),
            ..
        } = event
        {
            location_viewport.screen_pixel_width = width as u32;
            location_viewport.screen_pixel_height = height as u32;
            // continue to process other events that might be in the queue for this iteration
            continue;
        }

        // --- global escape handling ---
        if let Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } = event
        {
            let mut state_guard = game_state.lock().unwrap();
            match *state_guard {
                GameState::MainMenu => {
                    return Signal::Quit;
                }
                GameState::Playing => {
                    *state_guard = GameState::GameMenu;
                    controls.paused = true;
                }
                GameState::GameMenu => {
                    *state_guard = GameState::Playing;
                    controls.paused = false;
                }
                GameState::BuildMenu => {
                    *state_guard = GameState::Playing;
                }
                GameState::ShipyardMenu => {
                    *state_guard = GameState::Playing;
                }
                GameState::ShipyardMenuError { .. } => {
                    *state_guard = GameState::Playing;
                }
                GameState::Intro => {}
            }
            continue; // skip further processing for this Escape event
        }

        // --- state-specific handling ---
        let mut state_guard = game_state.lock().unwrap();
        let current_state_clone = state_guard.clone();

        match current_state_clone {
            GameState::MainMenu => {
                if let Some(signal) = main_menu::handle_main_menu_input(&event, &mut state_guard) {
                    return signal;
                }
            }
            GameState::Playing => {
                // always call playing::handle_playing_input when in GameState::Playing.
                // that function will decide what to do based on controls.paused.
                if let Some(signal) = playing::handle_playing_input(
                    &event,
                    mouse_pos,
                    location_viewport,
                    world,
                    controls,
                    &mut state_guard, // allows playing_input to change game_state (e.g. to BuildMenu)
                ) {
                    return signal; // e.g. if playing_input wants to quit
                }
            }
            GameState::GameMenu => {
                if let Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } = event
                {
                    return Signal::Quit;
                }
            }
            GameState::BuildMenu => {
                build_menu::handle_build_menu_input(
                    &event,
                    &current_state_clone,
                    world,
                    &controls.selection,
                    &mut state_guard,
                );
            }
            GameState::ShipyardMenu | GameState::ShipyardMenuError { .. } => {
                shipyard_menu::handle_shipyard_menu_input(
                    &event,
                    &current_state_clone,
                    world,
                    &controls.selection,
                    &mut state_guard,
                );
            }
            GameState::Intro => {}
        }
    }
    Signal::Continue
}
