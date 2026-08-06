use crate::command::Command;
use crate::infrastructure::InfrastructureCategory;
use crate::location::PointF64;
use crate::ships::{buildable_ship, ShipType};
use crate::world::components::{CivilianShipState, MiningRoute};
use crate::world::resources;
use crate::world::types::Storable;
use crate::world::{EntityId, World};
use rand::Rng;

pub type ShipSaleInfo = (EntityId, f64, EntityId, Vec<(Storable, f32)>);

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
            let layer = self
                .get_entity_type(entity_id)
                .and_then(crate::world::types::ConstructionLayer::primary_for)
                .unwrap_or(crate::world::types::ConstructionLayer::Orbit);
            if let Some(data) = self.celestial_data.get_mut(&entity_id) {
                if data.population <= 0.0 {
                    continue;
                }

                // resource consumption based on demand
                let mut demands: Vec<_> = data
                    .demands
                    .iter()
                    .map(|(storable, demand)| (*storable, *demand))
                    .collect();
                demands.sort_by_key(|(storable, _)| *storable);
                for (storable, monthly_demand) in demands {
                    // consumption is per second, so divide monthly demand by seconds in a month
                    const SECONDS_PER_MONTH: f64 = 30.0; // simplified
                    let consumption_rate = monthly_demand as f64 / SECONDS_PER_MONTH;
                    let total_consumption =
                        consumption_rate * data.population as f64 * dt / 1_000_000.0;

                    data.withdraw_at(layer, storable, total_consumption as f32);
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
                    if let Some(infrastructure) = self.infrastructure.get(&entity_id) {
                        let has_shipyard = infrastructure
                            .completed_units_in_category(InfrastructureCategory::Shipbuilding)
                            > 0;

                        let can_afford_ship_resources = buildable_ship(ShipType::MiningShip)
                            .is_some_and(|buildable| buildable.can_afford(data.stocks_at(layer)));

                        if has_shipyard && can_afford_ship_resources {
                            self.add_command(Command::BuildShip {
                                shipyard_entity_id: entity_id,
                                ship_type: ShipType::MiningShip,
                                civilian_credit_cost: Some(MINING_SHIP_COST),
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

        let mut potential_mining_targets: Vec<(u32, crate::world::types::RawResource)> = self
            .celestial_data
            .iter()
            .flat_map(|(id, data)| data.yields.keys().map(move |resource| (*id, *resource)))
            .collect();
        // sort for deterministic target order: this vec is built from hashmap
        // iteration and later indexed by a random roll, so its order matters.
        potential_mining_targets.sort_unstable();

        if potential_mining_targets.is_empty() {
            return;
        }

        // snapshot ship ai in a stable order so we can roll the world rng
        // (a mutable borrow) between the immutable decision calls.
        let mut ships: Vec<(
            EntityId,
            crate::world::components::CivilianShipAI,
            Option<MiningRoute>,
        )> = self
            .civilian_ai
            .iter()
            .map(|(&id, ai)| (id, ai.clone(), self.mining_routes.get(&id).copied()))
            .collect();
        ships.sort_by_key(|(id, _, _)| *id);

        for (ship_id, ai, route) in &ships {
            // pre-roll the target selection while we hold the rng mutably; the
            // decision function itself stays a pure function of world state.
            let target_roll = self.rng.0.random::<f64>();
            let (new_state, command) = self.decide_civilian_ship_action(
                *ship_id,
                ai,
                &potential_mining_targets,
                dt,
                self.enable_civilian_ai,
                route.as_ref(),
                target_roll,
            );
            if let Some(state) = new_state {
                state_changes_to_apply.push((*ship_id, state));
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
    #[allow(clippy::too_many_arguments)]
    fn decide_civilian_ship_action(
        &self,
        ship_id: EntityId,
        ai: &crate::world::components::CivilianShipAI,
        potential_mining_targets: &[(u32, crate::world::types::RawResource)],
        dt: f64,
        enable_random_ai: bool,
        route: Option<&MiningRoute>,
        // pre-rolled uniform value in [0, 1) used to pick a random in-range
        // target; passed in so this stays a pure function (no rng ownership).
        target_roll: f64,
    ) -> (Option<CivilianShipState>, Option<Command>) {
        let home_base = ai.home_base;

        match &ai.state {
            CivilianShipState::Idle => {
                let ship_pos = match self.get_location_f64(ship_id) {
                    Some(pos) => pos,
                    None => return (None, None),
                };

                if let Some(route) = route {
                    let target_id = route.target_body;
                    let resource = route.resource;
                    if let Some(intercept_pos) = predict_orbital_intercept(self, ship_id, target_id)
                    {
                        let command = Command::MoveShip {
                            ship_id,
                            destination: intercept_pos,
                        };
                        let new_state = CivilianShipState::MovingToMine {
                            target: target_id,
                            resource,
                        };
                        (Some(new_state), Some(command))
                    } else {
                        (None, None)
                    }
                } else if enable_random_ai {
                    let max_range_sq = self
                        .find_star_for_entity(home_base)
                        .map(|star_id| self.get_system_radius(star_id).powi(2))
                        .unwrap_or(100.0f64.powi(2));

                    let in_range_targets: Vec<(u32, crate::world::types::RawResource)> =
                        potential_mining_targets
                            .iter()
                            .filter_map(|&(target_id, resource)| {
                                if target_id == home_base {
                                    return None;
                                }
                                self.get_location_f64(target_id)
                                    .map(|target_pos| (target_id, resource, target_pos))
                            })
                            .filter(|(_, _, target_pos)| {
                                let dist_sq = (ship_pos.x - target_pos.x).powi(2)
                                    + (ship_pos.y - target_pos.y).powi(2);
                                dist_sq <= max_range_sq
                            })
                            .map(|(id, resource, _)| (id, resource))
                            .collect();

                    if !in_range_targets.is_empty() {
                        let index = ((target_roll * in_range_targets.len() as f64) as usize)
                            .min(in_range_targets.len() - 1);
                        let &(target_id, resource) = &in_range_targets[index];
                        if let Some(intercept_pos) =
                            predict_orbital_intercept(self, ship_id, target_id)
                        {
                            let command = Command::MoveShip {
                                ship_id,
                                destination: intercept_pos,
                            };
                            let new_state = CivilianShipState::MovingToMine {
                                target: target_id,
                                resource,
                            };
                            (Some(new_state), Some(command))
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            }
            CivilianShipState::MovingToMine { target, resource } => {
                if !self.move_orders.contains_key(&ship_id) {
                    (
                        Some(CivilianShipState::Mining {
                            target: *target,
                            resource: *resource,
                            mine_time: 0,
                        }),
                        None,
                    )
                } else {
                    (None, None)
                }
            }
            CivilianShipState::Mining {
                target,
                resource,
                mine_time,
            } => {
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
                                resource: *resource,
                                mine_time: mine_time + (dt * 1000.0) as u64,
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
        let mining_updates: Vec<(EntityId, EntityId, crate::world::types::RawResource)> = self
            .civilian_ai
            .iter()
            .filter_map(|(&ship_id, ai)| match ai.state {
                CivilianShipState::Mining {
                    target, resource, ..
                } => Some((ship_id, target, resource)),
                _ => None,
            })
            .collect();

        for (ship_id, target, resource) in mining_updates {
            if let (Some(cargo), Some(target_data)) = (
                self.cargo.get_mut(&ship_id),
                self.celestial_data.get(&target),
            ) {
                if let Some(yield_rate) = target_data.yields.get(&resource) {
                    const MINING_RATE: f32 = 1.0; // units per second
                    let mined_amount = yield_rate * MINING_RATE * dt as f32;
                    cargo.add(crate::world::types::Storable::Raw(resource), mined_amount);
                }
            }
        }
    }

    pub(super) fn process_ship_sales(&mut self) -> Vec<ShipSaleInfo> {
        let mut sales_updates: Vec<(EntityId, EntityId)> = self
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
        // sort by ship id so credits/stocks accumulate on the destination body in
        // a deterministic order; float addition isn't associative, and this feeds
        // build-timing decisions and thus the rng stream.
        sales_updates.sort_unstable();

        let mut sales_info: Vec<ShipSaleInfo> = Vec::new();
        let mut sales_to_process = Vec::new();

        for (ship_id, home_base_id) in sales_updates {
            if let Some(cargo) = self.cargo.get(&ship_id) {
                if cargo.current_load > 0.0 {
                    let mut drained_cargo = Vec::new();
                    for (storable, amount) in &cargo.contents {
                        drained_cargo.push((*storable, *amount));
                    }
                    drained_cargo.sort_by_key(|(storable, _)| *storable);
                    sales_to_process.push((ship_id, home_base_id, drained_cargo));
                }
            }
        }

        for (ship_id, home_base_id, drained_cargo) in sales_to_process {
            let prices: Vec<f64> = drained_cargo
                .iter()
                .map(|(storable, _)| resources::get_local_price(self, home_base_id, *storable))
                .collect();

            let layer = self
                .get_entity_type(home_base_id)
                .and_then(crate::world::types::ConstructionLayer::primary_for)
                .unwrap_or(crate::world::types::ConstructionLayer::Orbit);
            let storage_capacity = self
                .infrastructure
                .get(&home_base_id)
                .map(|infrastructure| infrastructure.storage_capacity(layer))
                .unwrap_or(0.0);
            if let Some(base_data) = self.celestial_data.get_mut(&home_base_id) {
                let mut total_value = 0.0;
                let mut accepted_cargo = Vec::new();
                for ((storable, amount), price) in drained_cargo.iter().zip(prices.iter()) {
                    let accepted =
                        base_data.deposit_bounded_at(layer, *storable, *amount, storage_capacity);
                    if accepted > 0.0 {
                        total_value += accepted as f64 * *price;
                        accepted_cargo.push((*storable, accepted));
                    }
                }
                if let Some(cargo) = self.cargo.get_mut(&ship_id) {
                    for (storable, accepted) in &accepted_cargo {
                        cargo.remove(*storable, *accepted);
                    }
                }
                base_data.credits += total_value;
                sales_info.push((ship_id, total_value, home_base_id, accepted_cargo));
            }
        }

        sales_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::EntityInfrastructure;
    use crate::location::Point;
    use crate::world::types::{CelestialBodyData, InfrastructureType};

    #[test]
    fn civilian_ai_keeps_credits_when_shipyard_lacks_mining_ship_resources() {
        let mut world = World::default();
        let shipyard_id = world.spawn_star("shipyard".to_string(), Point { x: 0, y: 0 });
        world.celestial_data.insert(
            shipyard_id,
            CelestialBodyData {
                credits: 1000.0,
                population: 1.0,
                ..Default::default()
            },
        );
        let mut infrastructure = EntityInfrastructure::new("shipyard");
        infrastructure.infra.insert(InfrastructureType::Shipyard, 1);
        world.infrastructure.insert(shipyard_id, infrastructure);

        world.update_civilian_economy(0.0);

        assert_eq!(world.celestial_data[&shipyard_id].credits, 1000.0);
        assert!(world.command_queue.is_empty());
    }

    #[test]
    fn delivery_accepts_only_free_storage_and_keeps_remainder_aboard() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let planet_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 0.0);
        world
            .infrastructure
            .get_mut(&planet_id)
            .unwrap()
            .infra
            .insert(InfrastructureType::SurfaceWarehouse, 1);
        world
            .celestial_data
            .get_mut(&planet_id)
            .unwrap()
            .deposit_bounded_at(
                crate::world::types::ConstructionLayer::Surface,
                Storable::Good(crate::world::types::Good::Food),
                995.0,
                1_000.0,
            );
        let ship_id = world.spawn_mining_ship("miner".to_string(), Point { x: 0, y: 0 }, planet_id);
        world.cargo.get_mut(&ship_id).unwrap().add(
            Storable::Raw(crate::world::types::RawResource::Metals),
            10.0,
        );

        let sales = world.process_ship_sales();

        assert_eq!(sales.len(), 1);
        assert_eq!(
            sales[0].3,
            vec![(Storable::Raw(crate::world::types::RawResource::Metals), 5.0)]
        );
        assert_eq!(world.cargo[&ship_id].current_load, 5.0);
        assert_eq!(
            world.celestial_data[&planet_id]
                .stored_units_at(crate::world::types::ConstructionLayer::Surface),
            1_000.0
        );
    }

    #[test]
    fn refinery_input_demand_does_not_accumulate_across_updates() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let planet_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 0.0);
        world.celestial_data.get_mut(&planet_id).unwrap().population = 1.0;
        world
            .infrastructure
            .get_mut(&planet_id)
            .unwrap()
            .infra
            .insert(InfrastructureType::FuelCellCracker, 1);
        let before = world.celestial_data[&planet_id].demands.clone();

        world.update_civilian_economy(0.0);
        world.update_civilian_economy(0.0);

        assert_eq!(world.celestial_data[&planet_id].demands, before);
        assert!(
            resources::get_local_price(
                &world,
                planet_id,
                Storable::Raw(crate::world::types::RawResource::Volatiles)
            ) > resources::get_resource_base_price(Storable::Raw(
                crate::world::types::RawResource::Volatiles
            ))
        );
    }
}
