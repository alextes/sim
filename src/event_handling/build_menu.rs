use crate::buildings::{BuildingType, SlotType};
use crate::world::World;
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_build_menu_input(
    event: &Event,
    current_state: &GameState, // Read current state for logic
    world: &mut World,         // Mutable world to potentially build
    entity_focus_index: usize, // Index is Copy, pass directly
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>, // Guard to modify state
) {
    match current_state {
        GameState::BuildMenuSelectingSlotType => {
            if let Event::KeyDown {
                keycode: Some(ref keycode),
                ..
            } = event
            {
                if world.entities.is_empty() {
                    // Should not happen if we entered build mode, but handle defensively
                    **game_state_guard = GameState::Playing;
                    return;
                }
                let selected_id = world.entities[entity_focus_index];
                match *keycode {
                    Keycode::G => {
                        if let Some(buildings) = world.buildings.get(&selected_id) {
                            if !buildings.has_ground_slots {
                                **game_state_guard = GameState::BuildMenuError {
                                    message: "cannot build on ground".to_string(),
                                };
                            } else if buildings.find_first_empty_slot(SlotType::Ground).is_some() {
                                **game_state_guard = GameState::BuildMenuSelectingBuilding {
                                    slot_type: SlotType::Ground,
                                };
                            } else {
                                **game_state_guard = GameState::BuildMenuError {
                                    message: "no ground space available".to_string(),
                                };
                            }
                        }
                    }
                    Keycode::O => {
                        if let Some(buildings) = world.buildings.get(&selected_id) {
                            if buildings.find_first_empty_slot(SlotType::Orbital).is_some() {
                                **game_state_guard = GameState::BuildMenuSelectingBuilding {
                                    slot_type: SlotType::Orbital,
                                };
                            } else {
                                **game_state_guard = GameState::BuildMenuError {
                                    message: "no orbital space available".to_string(),
                                };
                            }
                        }
                    }
                    _ => {} // Ignore other keys
                }
            }
        }
        GameState::BuildMenuSelectingBuilding { slot_type } => {
            let slot_type = *slot_type; // Deref the copy type
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
                    // Check placement rules again (although build method also checks)
                    let compatible = matches!(
                        (building, slot_type),
                        (BuildingType::SolarPanel, SlotType::Orbital)
                            | (BuildingType::Mine, SlotType::Ground)
                    );

                    if compatible {
                        if let Some(buildings_mut) = world.buildings.get_mut(&selected_id) {
                            if let Some(slot_idx) = buildings_mut.find_first_empty_slot(slot_type) {
                                match buildings_mut.build(slot_type, slot_idx, building) {
                                    Ok(_) => **game_state_guard = GameState::Playing, // Success!
                                    Err(msg) => {
                                        **game_state_guard = GameState::BuildMenuError {
                                            message: msg.to_string(),
                                        }
                                    }
                                }
                            } else {
                                // Should be unreachable due to check in previous state, but handle defensively
                                let err_msg = match slot_type {
                                    SlotType::Ground => "no ground space available".to_string(),
                                    SlotType::Orbital => "no orbital space available".to_string(),
                                };
                                **game_state_guard = GameState::BuildMenuError { message: err_msg };
                            }
                        }
                    } else {
                        let err_msg = match building {
                            BuildingType::SolarPanel => {
                                "solar panels require orbital slots".to_string()
                            }
                            BuildingType::Mine => "mines require ground slots".to_string(),
                        };
                        **game_state_guard = GameState::BuildMenuError { message: err_msg };
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
