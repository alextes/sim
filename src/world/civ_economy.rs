use crate::command::Command;
use crate::location::PointF64;
use crate::ships::ShipType;
use crate::world::components::CivilianShipState;
use crate::world::resources;
use crate::world::{EntityId, World};
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

        for entity_id in celestial_body_ids {
            if let Some(data) = self.celestial_data.get_mut(&entity_id) {
                if data.population <= 0.0 {
                    continue;
                }

                // resource consumption based on demand
                for (storable, &monthly_demand) in &data.demands {
                    // consumption is per second, so divide monthly demand by seconds in a month
                    const SECONDS_PER_MONTH: f64 = 30.0; // simplified
                    let consumption_rate = monthly_demand as f64 / SECONDS_PER_MONTH;
                    let total_consumption =
                        consumption_rate * data.population as f64 * dt / 1_000_000.0;

                    if let Some(stock) = data.stocks.get_mut(storable) {
                        let consumed_amount = (*stock as f64).min(total_consumption);

                        if consumed_amount > 0.0 {
                            *stock -= consumed_amount as f32;
                            // for now, consumption doesn't generate credits. that's what industry is for.
                        }
                    }
                }

                // decision to build a mining ship
                const MINING_SHIP_COST: f64 = 1000.0;
                const MAX_MINING_SHIPS_PER_BODY: usize = 64;

                // refinery demand
                if let Some(buildings) = self.buildings.get(&entity_id) {
                    let cracker_infra = buildings
                        .slots
                        .iter()
                        .filter(|s| {
                            s.is_some()
                                && s.unwrap() == crate::buildings::BuildingType::FuelCellCracker
                        })
                        .count() as f32;

                    if cracker_infra > 0.0 {
                        let demand_per_cracker = 10.0; // demand 10 units per month per cracker
                        data.demands
                            .entry(crate::world::types::Storable::Raw(
                                crate::world::types::RawResource::Volatiles,
                            ))
                            .and_modify(|d| *d += demand_per_cracker * cracker_infra)
                            .or_insert(demand_per_cracker * cracker_infra);
                    }
                }

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
        // --- stage 1: collect all ai decisions and required commands without mutating world ---
        let mut commands_to_issue = Vec::new();
        let mut state_changes_to_apply = Vec::new();

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
            let (new_state, command) =
                self.decide_civilian_ship_action(ship_id, ai, &potential_mining_targets, dt);
            if let Some(state) = new_state {
                state_changes_to_apply.push((ship_id, state));
            }
            if let Some(cmd) = command {
                commands_to_issue.push(cmd);
            }
        }

        // --- stage 2: apply all collected changes ---
        for (ship_id, new_state) in state_changes_to_apply {
            if let Some(ai) = self.civilian_ai.get_mut(&ship_id) {
                ai.state = new_state;
            }
        }

        for cmd in commands_to_issue {
            self.add_command(cmd);
        }
    }

    // this is now a pure function that returns decisions, not applying them.
    fn decide_civilian_ship_action(
        &self,
        ship_id: EntityId,
        ai: &crate::world::components::CivilianShipAI,
        potential_mining_targets: &[u32],
        dt: f64,
    ) -> (Option<CivilianShipState>, Option<Command>) {
        let home_base = ai.home_base;

        match &ai.state {
            CivilianShipState::Idle => {
                let ship_pos = match self.get_location_f64(ship_id) {
                    Some(pos) => pos,
                    None => return (None, None),
                };

                let max_range_sq = self
                    .find_star_for_entity(home_base)
                    .map(|star_id| self.get_system_radius(star_id).powi(2))
                    .unwrap_or(100.0f64.powi(2));

                let in_range_targets: Vec<u32> = potential_mining_targets
                    .iter()
                    .filter_map(|&target_id| {
                        if target_id == home_base {
                            return None;
                        }
                        self.get_location_f64(target_id)
                            .map(|target_pos| (target_id, target_pos))
                    })
                    .filter(|(_, target_pos)| {
                        let dist_sq = (ship_pos.x - target_pos.x).powi(2)
                            + (ship_pos.y - target_pos.y).powi(2);
                        dist_sq <= max_range_sq
                    })
                    .map(|(id, _)| id)
                    .collect();

                if !in_range_targets.is_empty() {
                    let target_id =
                        in_range_targets[rand::rng().random_range(0..in_range_targets.len())];
                    if let Some(intercept_pos) = predict_orbital_intercept(self, ship_id, target_id)
                    {
                        let command = Command::MoveShip {
                            ship_id,
                            destination: intercept_pos,
                        };
                        let new_state = CivilianShipState::MovingToMine { target: target_id };
                        (Some(new_state), Some(command))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            }
            CivilianShipState::MovingToMine { target } => {
                if !self.move_orders.contains_key(&ship_id) {
                    (
                        Some(CivilianShipState::Mining {
                            target: *target,
                            mine_time: 0.0,
                        }),
                        None,
                    )
                } else {
                    (None, None)
                }
            }
            CivilianShipState::Mining { target, mine_time } => {
                if let Some(cargo) = self.cargo.get(&ship_id) {
                    if cargo.current_load >= cargo.capacity {
                        if let Some(base_pos) = predict_orbital_intercept(self, ship_id, home_base)
                        {
                            let command = Command::MoveShip {
                                ship_id,
                                destination: base_pos,
                            };
                            (Some(CivilianShipState::ReturningToSell), Some(command))
                        } else {
                            (None, None)
                        }
                    } else {
                        (
                            Some(CivilianShipState::Mining {
                                target: *target,
                                mine_time: mine_time + dt,
                            }),
                            None,
                        )
                    }
                } else {
                    (None, None)
                }
            }
            CivilianShipState::ReturningToSell => {
                if !self.move_orders.contains_key(&ship_id) {
                    (Some(CivilianShipState::Idle), None)
                } else {
                    (None, None)
                }
            }
        }
    }

    pub(super) fn process_ship_mining(&mut self, dt: f64) {
        let mining_updates: Vec<(EntityId, EntityId)> = self
            .civilian_ai
            .iter()
            .filter_map(|(&ship_id, ai)| match ai.state {
                CivilianShipState::Mining { target, .. } => Some((ship_id, target)),
                _ => None,
            })
            .collect();

        for (ship_id, target) in mining_updates {
            if let (Some(cargo), Some(target_data)) = (
                self.cargo.get_mut(&ship_id),
                self.celestial_data.get(&target),
            ) {
                for (resource, yield_rate) in &target_data.yields {
                    const MINING_RATE: f32 = 1.0; // units per second
                    let mined_amount = yield_rate * MINING_RATE * dt as f32;
                    cargo.add(crate::world::types::Storable::Raw(*resource), mined_amount);
                }
            }
        }
    }

    pub(super) fn process_ship_sales(&mut self) -> Vec<(EntityId, f64, EntityId)> {
        let sales_updates: Vec<(EntityId, EntityId)> = self
            .civilian_ai
            .iter()
            .filter_map(|(id, ai)| {
                if ai.state == CivilianShipState::Idle {
                    if let Some(cargo) = self.cargo.get(id) {
                        if cargo.current_load > 0.0 {
                            return Some((*id, ai.home_base));
                        }
                    }
                }
                None
            })
            .collect();

        let mut sales_info = Vec::new();
        let mut sales_to_process = Vec::new();

        for (ship_id, home_base_id) in sales_updates {
            if let Some(cargo) = self.cargo.get(&ship_id) {
                if cargo.current_load > 0.0 {
                    let mut drained_cargo = Vec::new();
                    for (storable, amount) in &cargo.contents {
                        drained_cargo.push((*storable, *amount));
                    }
                    sales_to_process.push((ship_id, home_base_id, drained_cargo));
                }
            }
        }

        for (ship_id, home_base_id, drained_cargo) in sales_to_process {
            if let Some(cargo) = self.cargo.get_mut(&ship_id) {
                cargo.clear();
            }

            let prices: Vec<f64> = drained_cargo
                .iter()
                .map(|(storable, _)| resources::get_local_price(self, home_base_id, *storable))
                .collect();

            if let Some(base_data) = self.celestial_data.get_mut(&home_base_id) {
                let mut total_value = 0.0;
                for ((storable, amount), price) in drained_cargo.into_iter().zip(prices.into_iter())
                {
                    *base_data.stocks.entry(storable).or_insert(0.0) += amount;
                    total_value += amount as f64 * price;
                }
                base_data.credits += total_value;
                sales_info.push((ship_id, total_value, home_base_id));
            }
        }

        sales_info
    }
}
