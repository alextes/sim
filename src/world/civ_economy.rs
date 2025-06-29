use crate::command::Command;
use crate::location::PointF64;
use crate::ships::ShipType;
use crate::world::components::CivilianShipState;
use crate::world::resources;
use crate::world::types::Storable;
use crate::world::World;
use rand::Rng;

fn predict_orbital_intercept(world: &World, ship_id: u32, target_id: u32) -> Option<PointF64> {
    let ship_pos = world.get_location_f64(ship_id)?;
    let ship_speed = world.ships.get(&ship_id)?.speed;

    let orbital_params = match world.get_orbital_parameters(target_id) {
        Some(params) => params,
        None => return world.get_location_f64(target_id),
    };

    let anchor_pos = world.get_location_f64(orbital_params.anchor)?;
    let mut target_pos = world.get_location_f64(target_id)?;

    const ITERATIONS: u8 = 5;
    for _ in 0..ITERATIONS {
        let dist =
            ((ship_pos.x - target_pos.x).powi(2) + (ship_pos.y - target_pos.y).powi(2)).sqrt();
        if ship_speed <= 1e-6 {
            return Some(target_pos);
        }
        let time_to_intercept = dist / ship_speed;

        let future_angle =
            orbital_params.angle + orbital_params.angular_velocity * time_to_intercept;
        target_pos = PointF64 {
            x: anchor_pos.x + orbital_params.radius * future_angle.cos(),
            y: anchor_pos.y + orbital_params.radius * future_angle.sin(),
        };
    }

    Some(target_pos)
}

impl World {
    pub(super) fn update_civilian_economy(&mut self, dt: f64) {
        let celestial_body_ids: Vec<u32> = self.celestial_data.keys().cloned().collect();

        let consumption_rates = [
            (crate::world::types::RawResource::Metals, 0.001),
            (crate::world::types::RawResource::Organics, 0.0005),
        ];

        for entity_id in celestial_body_ids {
            if let Some(data) = self.celestial_data.get_mut(&entity_id) {
                if data.population <= 0.0 {
                    continue;
                }

                for &(resource, rate) in &consumption_rates {
                    let total_consumption = rate * data.population as f64 * dt;

                    let storable = crate::world::types::Storable::Raw(resource);
                    if let Some(stock) = data.stocks.get_mut(&storable) {
                        let consumed_amount = (*stock as f64).min(total_consumption);

                        if consumed_amount > 0.0 {
                            *stock -= consumed_amount as f32;
                            let credits_generated =
                                consumed_amount * resources::get_resource_base_price(storable);
                            data.credits += credits_generated;
                        }
                    }
                }

                // decision to build a mining ship
                const MINING_SHIP_COST: f64 = 1000.0;
                const MAX_MINING_SHIPS_PER_BODY: usize = 64;

                let existing_ships_for_base = self
                    .civilian_ai
                    .values()
                    .filter(|ai| ai.home_base == entity_id)
                    .count();

                if data.credits >= MINING_SHIP_COST
                    && existing_ships_for_base < MAX_MINING_SHIPS_PER_BODY
                {
                    if let Some(buildings) = self.buildings.get(&entity_id) {
                        let has_shipyard = buildings
                            .slots
                            .iter()
                            .any(|s| s == &Some(crate::buildings::BuildingType::Shipyard));

                        if has_shipyard {
                            data.credits -= MINING_SHIP_COST;
                            self.add_command(Command::BuildShip {
                                shipyard_entity_id: entity_id,
                                ship_type: ShipType::MiningShip,
                            });
                            tracing::info!(
                                "entity {} is building a mining ship",
                                self.get_entity_name(entity_id)
                                    .unwrap_or_else(|| "unknown".to_string())
                            );
                        }
                    }
                }
            }
        }
    }

    pub(super) fn update_civilian_ships(&mut self, dt: f64) {
        let mut commands = Vec::new();
        let mut ai_state_changes = Vec::new();
        let mut sales_info = Vec::new();

        let potential_mining_targets: Vec<u32> = self
            .celestial_data
            .iter()
            .filter(|(_, data)| !data.yields.is_empty())
            .map(|(id, _)| *id)
            .collect();

        if potential_mining_targets.is_empty() {
            return;
        }

        for (&ship_id, ai) in &self.civilian_ai {
            let current_state = &ai.state;
            let home_base = ai.home_base;

            match current_state {
                CivilianShipState::Idle => {
                    let ship_pos = match self.get_location_f64(ship_id) {
                        Some(pos) => pos,
                        None => continue, // ship has no position, can't do anything
                    };

                    let max_range_sq = self
                        .find_star_for_entity(home_base)
                        .map(|star_id| self.get_system_radius(star_id).powi(2))
                        .unwrap_or(100.0f64.powi(2)); // fallback range if system not found

                    let in_range_targets: Vec<(u32, crate::location::PointF64)> =
                        potential_mining_targets
                            .iter()
                            .filter_map(|&target_id| {
                                if target_id == home_base {
                                    return None;
                                } // don't mine from home
                                self.get_location_f64(target_id)
                                    .map(|target_pos| (target_id, target_pos))
                            })
                            .filter(|(_, target_pos)| {
                                let dist_sq = (ship_pos.x - target_pos.x).powi(2)
                                    + (ship_pos.y - target_pos.y).powi(2);
                                dist_sq <= max_range_sq
                            })
                            .collect();

                    if !in_range_targets.is_empty() {
                        let (target_id, _target_pos) =
                            in_range_targets[rand::rng().random_range(0..in_range_targets.len())];

                        if let Some(intercept_pos) =
                            predict_orbital_intercept(self, ship_id, target_id)
                        {
                            commands.push(Command::MoveShip {
                                ship_id,
                                destination: intercept_pos,
                            });
                            ai_state_changes.push((
                                ship_id,
                                CivilianShipState::MovingToMine { target: target_id },
                            ));
                        }
                    }
                }
                CivilianShipState::MovingToMine { target } => {
                    if !self.move_orders.contains_key(&ship_id) {
                        ai_state_changes.push((
                            ship_id,
                            CivilianShipState::Mining {
                                target: *target,
                                mine_time: 0.0,
                            },
                        ));
                    }
                }
                CivilianShipState::Mining { target, mine_time } => {
                    if let Some(cargo) = self.cargo.get(&ship_id) {
                        if cargo.current_load >= cargo.capacity {
                            if let Some(base_pos) =
                                predict_orbital_intercept(self, ship_id, home_base)
                            {
                                commands.push(Command::MoveShip {
                                    ship_id,
                                    destination: base_pos,
                                });
                                ai_state_changes
                                    .push((ship_id, CivilianShipState::ReturningToSell));
                            }
                        } else {
                            ai_state_changes.push((
                                ship_id,
                                CivilianShipState::Mining {
                                    target: *target,
                                    mine_time: mine_time + dt,
                                },
                            ));
                        }
                    }
                }
                CivilianShipState::ReturningToSell => {
                    if !self.move_orders.contains_key(&ship_id) {
                        ai_state_changes.push((ship_id, CivilianShipState::Idle));
                    }
                }
            }
        }

        for (ship_id, new_state) in ai_state_changes {
            if let Some(ai) = self.civilian_ai.get_mut(&ship_id) {
                ai.state = new_state;
            }
        }

        // process mining and selling, which require mutable access to multiple parts of world
        for (ship_id, ai) in self.civilian_ai.iter_mut() {
            match ai.state {
                CivilianShipState::Mining { target, .. } => {
                    if let (Some(cargo), Some(target_data)) = (
                        self.cargo.get_mut(ship_id),
                        self.celestial_data.get(&target),
                    ) {
                        for (resource, yield_rate) in &target_data.yields {
                            const MINING_RATE: f32 = 1.0; // units per second
                            let mined_amount = yield_rate * MINING_RATE * dt as f32;
                            cargo.add(crate::world::types::Storable::Raw(*resource), mined_amount);
                        }
                    }
                }
                CivilianShipState::Idle => {
                    // check if we just arrived from selling
                    if let Some(cargo) = self.cargo.get_mut(ship_id) {
                        if cargo.current_load > 0.0 {
                            if let Some(base_data) = self.celestial_data.get_mut(&ai.home_base) {
                                let mut total_value = 0.0;
                                for (storable, amount) in cargo.contents.drain() {
                                    *base_data.stocks.entry(storable).or_insert(0.0) += amount;
                                    if let crate::world::types::Storable::Raw(resource) = storable {
                                        total_value += amount as f64
                                            * resources::get_resource_base_price(Storable::Raw(
                                                resource,
                                            ));
                                    }
                                }
                                base_data.credits += total_value;
                                cargo.clear();
                                sales_info.push((*ship_id, total_value, ai.home_base));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for (ship_id, total_value, home_base) in sales_info {
            tracing::info!(
                "ship {} sold {:.2} credits worth of resources to {}",
                self.get_entity_name(ship_id)
                    .unwrap_or_else(|| "unknown".to_string()),
                total_value,
                self.get_entity_name(home_base)
                    .unwrap_or_else(|| "unknown".to_string()),
            );
        }

        for cmd in commands {
            self.add_command(cmd);
        }
    }
}
