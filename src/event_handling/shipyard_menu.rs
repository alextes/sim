use crate::ships::ShipType;
use crate::world::World;
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_shipyard_menu_input(
    event: &Event,
    current_state: &GameState,
    world: &mut World,
    entity_focus_index: Option<usize>,
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>,
) {
    let entity_focus_index = if let Some(index) = entity_focus_index {
        index
    } else {
        **game_state_guard = GameState::Playing;
        return;
    };

    match current_state {
        GameState::ShipyardMenu => {
            if let Event::KeyDown {
                keycode: Some(ref keycode),
                ..
            } = event
            {
                if world.entities.is_empty() {
                    **game_state_guard = GameState::Playing;
                    return;
                }
                let selected_id = world.entities[entity_focus_index];

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
