use crate::buildings::BuildingType;
use crate::world::World;
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_build_menu_input(
    event: &Event,
    current_state: &GameState,         // Read current state for logic
    world: &mut World,                 // Mutable world to potentially build
    entity_focus_index: Option<usize>, // Can be None if no entity is selected
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>, // Guard to modify state
) {
    // If no entity is selected, we can't be in a build menu. Revert to Playing state.
    let entity_focus_index = if let Some(index) = entity_focus_index {
        index
    } else {
        **game_state_guard = GameState::Playing;
        return;
    };

    if let GameState::BuildMenu = current_state {
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
