use crate::{world::World, BuildMenuMode, GameState};
use sdl2::{event::Event, keyboard::Keycode};
use strum::IntoEnumIterator;

pub fn handle_build_menu_input(
    event: &Event,
    world: &mut World,
    game_state_guard: &mut GameState,
    selected_id: Option<u32>,
) {
    let mut next_game_state: Option<GameState> = None;
    if let GameState::BuildMenu { mode } = game_state_guard {
        if let Event::KeyDown {
            keycode: Some(keycode),
            ..
        } = event
        {
            match mode {
                BuildMenuMode::Main => match *keycode {
                    Keycode::Q => next_game_state = Some(GameState::Playing),
                    Keycode::A => {
                        next_game_state = Some(GameState::BuildMenu {
                            mode: BuildMenuMode::SelectBuilding,
                        })
                    }
                    _ => {}
                },
                BuildMenuMode::SelectBuilding => {
                    let building_type = match *keycode {
                        Keycode::Num1 => {
                            Some(crate::world::types::BuildingType::iter().next().unwrap())
                        }
                        Keycode::Num2 => {
                            Some(crate::world::types::BuildingType::iter().nth(1).unwrap())
                        }
                        Keycode::Num3 => {
                            Some(crate::world::types::BuildingType::iter().nth(2).unwrap())
                        }
                        Keycode::Num4 => {
                            Some(crate::world::types::BuildingType::iter().nth(3).unwrap())
                        }
                        Keycode::Num5 => {
                            Some(crate::world::types::BuildingType::iter().nth(4).unwrap())
                        }
                        Keycode::Num6 => {
                            Some(crate::world::types::BuildingType::iter().nth(5).unwrap())
                        }
                        _ => None,
                    };

                    if let Some(building) = building_type {
                        next_game_state = Some(GameState::BuildMenu {
                            mode: BuildMenuMode::EnterQuantity {
                                building,
                                quantity_string: String::new(),
                            },
                        });
                    }
                }
                BuildMenuMode::EnterQuantity {
                    building,
                    quantity_string,
                } => {
                    let mut new_quantity_string = quantity_string.clone();
                    match *keycode {
                        Keycode::Num0
                        | Keycode::Num1
                        | Keycode::Num2
                        | Keycode::Num3
                        | Keycode::Num4
                        | Keycode::Num5
                        | Keycode::Num6
                        | Keycode::Num7
                        | Keycode::Num8
                        | Keycode::Num9 => {
                            new_quantity_string.push(keycode.to_string().chars().last().unwrap());
                        }
                        Keycode::Backspace => {
                            new_quantity_string.pop();
                        }
                        Keycode::Return => {
                            if let Ok(amount) = new_quantity_string.parse::<u32>() {
                                if amount > 0 {
                                    next_game_state = Some(GameState::BuildMenu {
                                        mode: BuildMenuMode::ConfirmQuote {
                                            building: *building,
                                            amount,
                                        },
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                    if &new_quantity_string != quantity_string {
                        next_game_state = Some(GameState::BuildMenu {
                            mode: BuildMenuMode::EnterQuantity {
                                building: *building,
                                quantity_string: new_quantity_string,
                            },
                        });
                    }
                }
                BuildMenuMode::ConfirmQuote { building, amount } => match *keycode {
                    Keycode::Y => {
                        if let Some(entity_id) = selected_id {
                            world.add_command(crate::command::Command::Build {
                                entity_id,
                                building_type: *building,
                                amount: *amount,
                            });
                        }
                        next_game_state = Some(GameState::Playing);
                    }
                    Keycode::N => {
                        next_game_state = Some(GameState::BuildMenu {
                            mode: BuildMenuMode::Main,
                        });
                    }
                    _ => {}
                },
            }
        }
    }
    if let Some(next_state) = next_game_state {
        *game_state_guard = next_state;
    }
}
