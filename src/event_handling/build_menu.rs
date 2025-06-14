use crate::buildings::BuildingType;
use crate::world::{EntityId, World};
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_build_menu_input(
    event: &Event,
    current_state: &GameState, // Read current state for logic
    world: &mut World,         // Mutable world to potentially build
    selection: &[EntityId],
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>, // Guard to modify state
) {
    // If not exactly one entity is selected, we can't be in a build menu.
    if selection.len() != 1 {
        **game_state_guard = GameState::Playing;
        return;
    };
    let selected_id = selection[0];

    if let GameState::BuildMenu = current_state {
        if let Event::KeyDown {
            keycode: Some(ref keycode),
            ..
        } = event
        {
            let building_to_build = match *keycode {
                Keycode::Num1 => Some(BuildingType::SolarPanel),
                Keycode::Num2 => Some(BuildingType::Mine),
                _ => None,
            };

            if let Some(building) = building_to_build {
                world.add_command(crate::command::Command::BuildBuilding {
                    entity_id: selected_id,
                    building_type: building,
                });
                **game_state_guard = GameState::Playing;
            }
        }
    }
}
