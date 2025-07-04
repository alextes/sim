use crate::{event_handling::Signal, GameState};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::sync::MutexGuard;

pub fn handle_main_menu_input(
    event: &Event,
    state_guard: &mut MutexGuard<GameState>,
) -> Option<Signal> {
    if let Event::KeyDown {
        keycode: Some(keycode),
        ..
    } = event
    {
        match *keycode {
            Keycode::Return | Keycode::Space | Keycode::P => {
                **state_guard = GameState::Playing;
            }
            Keycode::Q => {
                return Some(Signal::Quit);
            }
            _ => {}
        }
    }
    None
}
