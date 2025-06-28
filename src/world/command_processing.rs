use crate::command::Command;
use crate::location::Point;
use crate::location::PointF64;
use crate::ships::ShipType;
use crate::world::World;
use rand::Rng;

impl World {
    pub(super) fn process_commands(&mut self) {
        let commands: Vec<Command> = self.command_queue.drain(..).collect();
        for command in commands {
            self.process_command(command);
        }
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::MoveShip {
                ship_id,
                destination,
            } => {
                self.set_move_order(ship_id, destination);
            }
            Command::BuildShip {
                shipyard_entity_id,
                ship_type,
            } => {
                if let Some(shipyard_loc) = self.locations.get_location(shipyard_entity_id) {
                    let mut rng = rand::rng();
                    let spawn_offset_x = rng.random_range(-2.0..2.0);
                    let spawn_offset_y = rng.random_range(-2.0..2.0);
                    let spawn_pos = PointF64 {
                        x: shipyard_loc.x as f64 + spawn_offset_x,
                        y: shipyard_loc.y as f64 + spawn_offset_y,
                    };
                    match ship_type {
                        ShipType::Frigate => {
                            self.spawn_frigate(
                                "new_frigate".to_string(),
                                Point {
                                    x: spawn_pos.x as i32,
                                    y: spawn_pos.y as i32,
                                },
                            );
                        }
                        ShipType::MiningShip => {
                            self.spawn_mining_ship(
                                "mining_ship".to_string(),
                                Point {
                                    x: spawn_pos.x as i32,
                                    y: spawn_pos.y as i32,
                                },
                                shipyard_entity_id, // home base
                            );
                        }
                    }
                }
            }
            Command::BuildBuilding {
                entity_id,
                building_type,
            } => {
                let costs = building_type.cost();
                let mut can_afford = true;

                if let Some(celestial_data) = self.celestial_data.get_mut(&entity_id) {
                    for (resource, cost) in &costs {
                        let stock = celestial_data.stocks.entry(*resource).or_insert(0.0);
                        if *stock < *cost {
                            can_afford = false;
                            tracing::warn!(
                                "cannot build {:?}, not enough {:?} (need {}, have {})",
                                building_type,
                                resource,
                                cost,
                                stock
                            );
                            break;
                        }
                    }

                    if can_afford {
                        if let Some(buildings) = self.buildings.get_mut(&entity_id) {
                            if let Some(slot) = buildings.find_first_empty_slot() {
                                if buildings.build(slot, building_type).is_ok() {
                                    // Deduct costs only after successfully building
                                    for (resource, cost) in &costs {
                                        if let Some(stock) = celestial_data.stocks.get_mut(resource)
                                        {
                                            *stock -= *cost;
                                        }
                                    }
                                    tracing::info!(
                                        "built {:?} on entity {}",
                                        building_type,
                                        entity_id
                                    );
                                }
                            }
                        }
                    }
                } else {
                    tracing::warn!("tried to build on an entity with no celestial data");
                }
            }
        }
    }
}
