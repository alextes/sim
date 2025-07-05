use crate::buildings::EntityBuildings;
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
            Command::Build {
                entity_id,
                building_type,
                amount,
            } => {
                let costs = EntityBuildings::get_build_costs(building_type, amount);
                let can_afford = {
                    if let Some(cd) = self.celestial_data.get(&entity_id) {
                        costs.iter().all(|(resource, &cost)| {
                            let stock = cd.stocks.get(resource).copied().unwrap_or(0.0);
                            if stock < cost {
                                tracing::warn!(
                                    "entity {} cannot afford to build {:?} x{}: not enough {:?} (needs {}, has {})",
                                    self.get_entity_name(entity_id).unwrap_or_default(),
                                    building_type,
                                    amount,
                                    resource,
                                    cost,
                                    stock
                                );
                                false
                            } else {
                                true
                            }
                        })
                    } else {
                        false
                    }
                };

                if can_afford {
                    if let Some(cd) = self.celestial_data.get_mut(&entity_id) {
                        for (resource, cost) in costs {
                            *cd.stocks.entry(resource).or_insert(0.0) -= cost;
                        }
                    }

                    if let Some(buildings) = self.buildings.get_mut(&entity_id) {
                        buildings.queue_build(building_type, amount);
                        tracing::info!(
                            "queued build of {:?} x{} on entity {}",
                            building_type,
                            amount,
                            self.get_entity_name(entity_id).unwrap_or_default()
                        );
                    }
                }
            }
        }
    }
}
