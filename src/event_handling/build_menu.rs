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

    match current_state {
        GameState::BuildMenu => {
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
                    if let Some(buildings_mut) = world.buildings.get_mut(&selected_id) {
                        if buildings_mut.slots.is_empty() {
                            **game_state_guard = GameState::BuildMenuError {
                                message: "this entity cannot have buildings.".to_string(),
                            };
                            return;
                        }

                        if let Some(slot_idx) = buildings_mut.find_first_empty_slot() {
                            match buildings_mut.build(slot_idx, building) {
                                Ok(_) => **game_state_guard = GameState::Playing, // Success!
                                Err(msg) => {
                                    **game_state_guard = GameState::BuildMenuError {
                                        message: msg.to_string(),
                                    }
                                }
                            }
                        } else {
                            **game_state_guard = GameState::BuildMenuError {
                                message: "no space available".to_string(),
                            };
                        }
                    } else {
                        // This case should not be reachable if we entered BuildMenu state,
                        // but handle defensively.
                        **game_state_guard = GameState::BuildMenuError {
                            message: "selected entity does not support buildings.".to_string(),
                        };
                    }
                }
            }
        }
        GameState::BuildMenuError { .. } => {
            // Any key press returns to playing state
            if let Event::KeyDown { .. } = event {
                **game_state_guard = GameState::Playing;
            }
        }
        _ => {} // Ignore GameState::Playing as it's handled elsewhere
    }
}
