use crate::world::types::RawResource;
use crate::world::{EntityId, World};
use crate::GameState;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn handle_mining_route_menu_input(
    event: &Event,
    current_state: &GameState,
    world: &mut World,
    _selection: &[EntityId],
    game_state_guard: &mut std::sync::MutexGuard<'_, GameState>,
) {
    let (ship_id, mode) = match current_state {
        GameState::MiningRouteMenu { ship_id, mode } => (*ship_id, mode.clone()),
        _ => return,
    };

    if let Event::KeyDown {
        keycode: Some(key), ..
    } = event
    {
        match mode {
            crate::MiningRouteMenuMode::SelectTarget => match *key {
                Keycode::Escape => {
                    **game_state_guard = GameState::Playing;
                }
                Keycode::A => {
                    if let Some(route) = world.compute_best_mining_route() {
                        world.set_mining_route(ship_id, Some(route));
                        **game_state_guard = GameState::Playing;
                    }
                }
                _ => {
                    if let Some(idx) = key_to_digit_index(*key) {
                        if let Some(target_id) = nth_body(world, idx) {
                            **game_state_guard = GameState::MiningRouteMenu {
                                ship_id,
                                mode: crate::MiningRouteMenuMode::SelectResource { target_id },
                            };
                        }
                    }
                }
            },
            crate::MiningRouteMenuMode::SelectResource { target_id } => match *key {
                Keycode::Escape => {
                    **game_state_guard = GameState::MiningRouteMenu {
                        ship_id,
                        mode: crate::MiningRouteMenuMode::SelectTarget,
                    };
                }
                _ => {
                    let raws: Vec<RawResource> = world
                        .celestial_data
                        .get(&target_id)
                        .map(|d| d.yields.keys().cloned().collect())
                        .unwrap_or_default();
                    if let Some(idx) = key_to_digit_index(*key) {
                        if let Some(&resource) = raws.get(idx) {
                            **game_state_guard = GameState::MiningRouteMenu {
                                ship_id,
                                mode: crate::MiningRouteMenuMode::SelectSell {
                                    target_id,
                                    resource,
                                },
                            };
                        }
                    }
                }
            },
            crate::MiningRouteMenuMode::SelectSell {
                target_id,
                resource,
            } => match *key {
                Keycode::Escape => {
                    **game_state_guard = GameState::MiningRouteMenu {
                        ship_id,
                        mode: crate::MiningRouteMenuMode::SelectResource { target_id },
                    };
                }
                _ => {
                    if let Some(idx) = key_to_digit_index(*key) {
                        if let Some(sell_id) = nth_body(world, idx) {
                            world.set_mining_route(
                                ship_id,
                                Some(crate::world::components::MiningRoute {
                                    target_body: target_id,
                                    resource,
                                    sell_body: sell_id,
                                }),
                            );
                            **game_state_guard = GameState::Playing;
                        }
                    }
                }
            },
        }
    }
}

fn nth_body(world: &World, index: usize) -> Option<EntityId> {
    let mut list: Vec<EntityId> = world
        .iter_entities()
        .filter(|id| world.celestial_data.contains_key(id))
        .collect();
    list.sort();
    list.get(index).cloned()
}

fn key_to_digit_index(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0),
        Keycode::Num2 => Some(1),
        Keycode::Num3 => Some(2),
        Keycode::Num4 => Some(3),
        Keycode::Num5 => Some(4),
        Keycode::Num6 => Some(5),
        Keycode::Num7 => Some(6),
        Keycode::Num8 => Some(7),
        Keycode::Num9 => Some(8),
        Keycode::Num0 => Some(9),
        _ => None,
    }
}
