use crate::ships::ShipType;
use crate::world::{EntityId, World};
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_shipyard_menu_input(
    event: &Event,
    current_state: &GameState,
    world: &mut World,
    selection: &[EntityId],
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>,
) {
    if selection.len() != 1 {
        **game_state_guard = GameState::Playing;
        return;
    }
    let selected_id = selection[0];

    match current_state {
        GameState::ShipyardMenu => {
            if let Event::KeyDown {
                keycode: Some(ref keycode),
                ..
            } = event
            {
                if *keycode == Keycode::Num1 {
                    world.add_command(crate::command::Command::BuildShip {
                        shipyard_entity_id: selected_id,
                        ship_type: ShipType::Frigate,
                    });
                    **game_state_guard = GameState::Playing;
                }
            }
        }
        GameState::ShipyardMenuError { .. } => {
            if let Event::KeyDown { .. } = event {
                **game_state_guard = GameState::Playing;
            }
        }
        _ => {}
    }
}
