use crate::command::Command;
use crate::infrastructure::EntityInfrastructure;
use crate::location::Point;
use crate::location::PointF64;
use crate::ships::{buildable_ship, ShipType};
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
                civilian_credit_cost,
            } => {
                let Some(buildable) = buildable_ship(ship_type) else {
                    tracing::warn!(
                        "entity {} cannot build unavailable ship {:?}",
                        self.get_entity_name(shipyard_entity_id).unwrap_or_default(),
                        ship_type
                    );
                    return;
                };

                let Some(shipyard_data) = self.celestial_data.get(&shipyard_entity_id) else {
                    tracing::warn!(
                        "entity {} cannot build ship {:?}: missing shipyard stocks",
                        self.get_entity_name(shipyard_entity_id).unwrap_or_default(),
                        ship_type
                    );
                    return;
                };

                let shortfall = buildable.first_shortfall(&shipyard_data.stocks);
                if let Some(shortfall) = shortfall {
                    tracing::warn!(
                        "entity {} cannot afford ship {:?}: not enough {:?} (needs {}, has {})",
                        self.get_entity_name(shipyard_entity_id).unwrap_or_default(),
                        ship_type,
                        shortfall.resource,
                        shortfall.required,
                        shortfall.available
                    );
                    return;
                }

                let Some(shipyard_loc) = self.locations.get_location(shipyard_entity_id) else {
                    return;
                };

                if let Some(credit_cost) = civilian_credit_cost {
                    let credits = self
                        .celestial_data
                        .get(&shipyard_entity_id)
                        .map(|cd| cd.credits)
                        .unwrap_or(0.0);
                    if credits < credit_cost {
                        tracing::warn!(
                            "entity {} cannot afford ship {:?}: not enough credits (needs {}, has {})",
                            self.get_entity_name(shipyard_entity_id).unwrap_or_default(),
                            ship_type,
                            credit_cost,
                            credits
                        );
                        return;
                    }
                }

                // deduct the cost from the shipyard body's stocks.
                if let Some(cd) = self.celestial_data.get_mut(&shipyard_entity_id) {
                    for cost in buildable.costs {
                        *cd.stocks.entry(cost.resource).or_insert(0.0) -= cost.quantity;
                    }
                    if let Some(credit_cost) = civilian_credit_cost {
                        cd.credits -= credit_cost;
                    }
                }

                let spawn_offset_x = self.rng.0.random_range(-2.0..2.0);
                let spawn_offset_y = self.rng.0.random_range(-2.0..2.0);
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
                            shipyard_entity_id,
                        );
                    }
                }
            }
            Command::Build {
                entity_id,
                infrastructure_type,
                amount,
            } => {
                let costs = EntityInfrastructure::get_build_costs(infrastructure_type, amount);
                let can_afford = {
                    if let Some(cd) = self.celestial_data.get(&entity_id) {
                        costs.iter().all(|(resource, &cost)| {
                            let stock = cd.stocks.get(resource).copied().unwrap_or(0.0);
                            if stock < cost {
                                tracing::warn!(
                                    "entity {} cannot afford to build {:?} x{}: not enough {:?} (needs {}, has {})",
                                    self.get_entity_name(entity_id).unwrap_or_default(),
                                    infrastructure_type,
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

                    if let Some(infrastructure) = self.infrastructure.get_mut(&entity_id) {
                        infrastructure.queue_build(infrastructure_type, amount);
                        tracing::info!(
                            "queued build of {:?} x{} on entity {}",
                            infrastructure_type,
                            amount,
                            self.get_entity_name(entity_id).unwrap_or_default()
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::location::Point;
    use crate::world::types::{CelestialBodyData, RawResource, Storable};
    use std::collections::HashMap;

    #[test]
    fn rejected_ship_build_does_not_deduct_resources_or_spawn_ship() {
        let mut world = World::default();
        let shipyard_id = world.spawn_star("shipyard".to_string(), Point { x: 0, y: 0 });
        world.celestial_data.insert(
            shipyard_id,
            CelestialBodyData {
                stocks: HashMap::from([(Storable::Raw(RawResource::Metals), 80.0)]),
                ..Default::default()
            },
        );

        world.add_command(Command::BuildShip {
            shipyard_entity_id: shipyard_id,
            ship_type: ShipType::Frigate,
            civilian_credit_cost: Some(1000.0),
        });
        world.process_commands();

        let stocks = &world.celestial_data[&shipyard_id].stocks;
        assert_eq!(stocks[&Storable::Raw(RawResource::Metals)], 80.0);
        assert_eq!(world.celestial_data[&shipyard_id].credits, 0.0);
        assert!(world.ships.is_empty());
    }

    #[test]
    fn accepted_civilian_ship_build_deducts_resources_and_credits() {
        let mut world = World::default();
        let shipyard_id = world.spawn_star("shipyard".to_string(), Point { x: 0, y: 0 });
        world.celestial_data.insert(
            shipyard_id,
            CelestialBodyData {
                credits: 1000.0,
                stocks: HashMap::from([
                    (Storable::Raw(RawResource::Metals), 50.0),
                    (Storable::Raw(RawResource::Crystals), 15.0),
                ]),
                ..Default::default()
            },
        );

        world.add_command(Command::BuildShip {
            shipyard_entity_id: shipyard_id,
            ship_type: ShipType::MiningShip,
            civilian_credit_cost: Some(1000.0),
        });
        world.process_commands();

        let stocks = &world.celestial_data[&shipyard_id].stocks;
        assert_eq!(stocks[&Storable::Raw(RawResource::Metals)], 0.0);
        assert_eq!(stocks[&Storable::Raw(RawResource::Crystals)], 0.0);
        assert_eq!(world.celestial_data[&shipyard_id].credits, 0.0);
        assert_eq!(world.ships.len(), 1);
    }
}
