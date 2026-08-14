use crate::command::Command;
use crate::infrastructure::InfrastructureCategory;
use crate::location::PointF64;
use crate::ships::{buildable_ship, ShipType};
use crate::world::components::{CivilianShipState, MiningRoute, MiningRouteEstimate};
use crate::world::types::{ConstructionLayer, ProcurementKey, Storable};
use crate::world::{EntityId, World};

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

fn point_distance(a: PointF64, b: PointF64) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

impl World {
    pub fn estimate_mining_route(
        &self,
        ship_id: EntityId,
        route: MiningRoute,
    ) -> Option<MiningRouteEstimate> {
        let ship = self.ships.get(&ship_id)?;
        let cargo_capacity = self.cargo.get(&ship_id)?.capacity;
        let mining_yield = self
            .celestial_data
            .get(&route.target_body)?
            .yields
            .get(&route.resource)
            .copied()?;
        let quote = self.procurement_quote(
            route.sell_body,
            ProcurementKey {
                layer: ConstructionLayer::Orbit,
                resource: Storable::Raw(route.resource),
            },
        )?;
        let sale_quantity = cargo_capacity.min(quote.wanted_quantity);
        let mining_rate = mining_yield * self.civilian_economy_config.mining_units_per_yield_second;
        if sale_quantity <= 0.0 || mining_rate <= 0.0 || ship.speed <= 0.0 {
            return None;
        }

        let ship_position = self.get_location_f64(ship_id)?;
        let target_position = self.get_location_f64(route.target_body)?;
        let sell_position = self.get_location_f64(route.sell_body)?;
        let outbound_distance = point_distance(ship_position, target_position);
        let delivery_distance = point_distance(target_position, sell_position);
        let travel_distance = outbound_distance + delivery_distance;
        let travel_time = travel_distance / ship.speed;
        let mining_time = sale_quantity as f64 / mining_rate as f64;
        let cycle_time = travel_time + mining_time;
        if cycle_time <= 0.0 || !cycle_time.is_finite() {
            return None;
        }

        let config = self.civilian_economy_config;
        let operating_cost = travel_distance * config.travel_cost_per_distance
            + mining_time * config.mining_cost_per_second
            + config.ship_maintenance_per_cycle
            + config.docking_fee;
        let sale_revenue = sale_quantity as f64 * quote.unit_price;
        let expected_profit = sale_revenue - operating_cost;

        Some(MiningRouteEstimate {
            route,
            sale_quantity,
            mining_yield,
            unit_price: quote.unit_price,
            sale_revenue,
            travel_distance,
            travel_time,
            mining_time,
            operating_cost,
            cycle_time,
            expected_profit,
            profit_per_second: expected_profit / cycle_time,
        })
    }

    pub fn best_mining_opportunity(&self, ship_id: EntityId) -> Option<MiningRouteEstimate> {
        let mut procurement_opportunities = Vec::new();
        for (&sell_id, body) in &self.celestial_data {
            for &key in body.procurement_policies.keys() {
                let Storable::Raw(resource) = key.resource else {
                    continue;
                };
                if key.layer == ConstructionLayer::Orbit
                    && self.procurement_quote(sell_id, key).is_some()
                {
                    procurement_opportunities.push((resource, sell_id));
                }
            }
        }
        procurement_opportunities.sort_unstable();
        if procurement_opportunities.is_empty() {
            return None;
        }

        let mut source_ids: Vec<_> = self.celestial_data.keys().copied().collect();
        source_ids.sort_unstable();
        let mut best = None;

        for source_id in source_ids {
            let mut resources: Vec<_> = self.celestial_data[&source_id]
                .yields
                .keys()
                .copied()
                .collect();
            resources.sort_unstable();
            for resource in resources {
                for &(_, sell_id) in procurement_opportunities
                    .iter()
                    .filter(|(wanted_resource, _)| *wanted_resource == resource)
                {
                    if source_id == sell_id {
                        continue;
                    }
                    let route = MiningRoute {
                        target_body: source_id,
                        resource,
                        sell_body: sell_id,
                    };
                    let Some(estimate) = self.estimate_mining_route(ship_id, route) else {
                        continue;
                    };
                    if estimate.expected_profit <= 0.0 {
                        continue;
                    }
                    let improves_score =
                        best.as_ref().is_none_or(|current: &MiningRouteEstimate| {
                            estimate.profit_per_second > current.profit_per_second
                        });
                    if improves_score {
                        best = Some(estimate);
                    }
                }
            }
        }
        best
    }

    pub fn compute_best_mining_route(&self, ship_id: EntityId) -> Option<MiningRoute> {
        self.best_mining_opportunity(ship_id)
            .map(|estimate| estimate.route)
    }

    pub fn active_mining_route(&self, ship_id: EntityId) -> Option<MiningRoute> {
        let ai = self.civilian_ai.get(&ship_id)?;
        match ai.state {
            CivilianShipState::Idle => self.mining_routes.get(&ship_id).copied(),
            CivilianShipState::MovingToMine { route, .. }
            | CivilianShipState::Mining { route, .. }
            | CivilianShipState::ReturningToSell { route, .. }
            | CivilianShipState::WaitingToUnload { route, .. } => Some(route),
        }
    }

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
                            .operational_units_in_category(InfrastructureCategory::Shipbuilding)
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

        // snapshot ship ai in stable order before applying any decisions.
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
            let (new_state, command) = self.decide_civilian_ship_action(
                *ship_id,
                ai,
                dt,
                self.enable_civilian_ai,
                route.as_ref(),
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
    fn decide_civilian_ship_action(
        &self,
        ship_id: EntityId,
        ai: &crate::world::components::CivilianShipAI,
        dt: f64,
        enable_autonomous_ai: bool,
        route: Option<&MiningRoute>,
    ) -> (Option<CivilianShipState>, Option<Command>) {
        match &ai.state {
            CivilianShipState::Idle => {
                let Some(cargo) = self.cargo.get(&ship_id) else {
                    return (None, None);
                };
                let selected_trip = if let Some(&manual_route) = route {
                    let cargo_target = self
                        .estimate_mining_route(ship_id, manual_route)
                        .map(|estimate| estimate.sale_quantity)
                        .unwrap_or(cargo.capacity);
                    Some((manual_route, cargo_target))
                } else if enable_autonomous_ai {
                    self.best_mining_opportunity(ship_id)
                        .map(|estimate| (estimate.route, estimate.sale_quantity))
                } else {
                    None
                };
                let Some((route, cargo_target)) = selected_trip else {
                    return (None, None);
                };
                let cargo_target = cargo_target.max(cargo.current_load).min(cargo.capacity);
                let Some(intercept_pos) =
                    predict_orbital_intercept(self, ship_id, route.target_body)
                else {
                    return (None, None);
                };
                let command = Command::MoveShip {
                    ship_id,
                    destination: intercept_pos,
                };
                (
                    Some(CivilianShipState::MovingToMine {
                        route,
                        cargo_target,
                    }),
                    Some(command),
                )
            }
            CivilianShipState::MovingToMine {
                route,
                cargo_target,
            } => {
                if !self.move_orders.contains_key(&ship_id) {
                    (
                        Some(CivilianShipState::Mining {
                            route: *route,
                            cargo_target: *cargo_target,
                            mine_time: 0,
                        }),
                        None,
                    )
                } else {
                    (None, None)
                }
            }
            CivilianShipState::Mining {
                route,
                cargo_target,
                mine_time,
            } => {
                if let Some(cargo) = self.cargo.get(&ship_id) {
                    if cargo.current_load >= *cargo_target {
                        if let Some(base_pos) =
                            predict_orbital_intercept(self, ship_id, route.sell_body)
                        {
                            let command = Command::MoveShip {
                                ship_id,
                                destination: base_pos,
                            };
                            (
                                Some(CivilianShipState::ReturningToSell {
                                    route: *route,
                                    cargo_target: *cargo_target,
                                }),
                                Some(command),
                            )
                        } else {
                            (None, None)
                        }
                    } else {
                        (
                            Some(CivilianShipState::Mining {
                                route: *route,
                                cargo_target: *cargo_target,
                                mine_time: mine_time + (dt * 1000.0) as u64,
                            }),
                            None,
                        )
                    }
                } else {
                    (None, None)
                }
            }
            CivilianShipState::ReturningToSell {
                route,
                cargo_target,
            } => {
                if !self.move_orders.contains_key(&ship_id) {
                    (
                        Some(CivilianShipState::WaitingToUnload {
                            route: *route,
                            cargo_target: *cargo_target,
                        }),
                        None,
                    )
                } else {
                    (None, None)
                }
            }
            CivilianShipState::WaitingToUnload { .. } => (None, None),
        }
    }

    pub(super) fn process_ship_mining(&mut self, dt: f64) {
        let mining_updates: Vec<(EntityId, EntityId, crate::world::types::RawResource, f32)> = self
            .civilian_ai
            .iter()
            .filter_map(|(&ship_id, ai)| match ai.state {
                CivilianShipState::Mining {
                    route,
                    cargo_target,
                    ..
                } => Some((ship_id, route.target_body, route.resource, cargo_target)),
                _ => None,
            })
            .collect();

        for (ship_id, target, resource, cargo_target) in mining_updates {
            if let (Some(cargo), Some(target_data)) = (
                self.cargo.get_mut(&ship_id),
                self.celestial_data.get(&target),
            ) {
                if let Some(yield_rate) = target_data.yields.get(&resource) {
                    let mined_amount = yield_rate
                        * self.civilian_economy_config.mining_units_per_yield_second
                        * dt as f32;
                    let remaining = (cargo_target - cargo.current_load).max(0.0);
                    cargo.add(
                        crate::world::types::Storable::Raw(resource),
                        mined_amount.min(remaining),
                    );
                }
            }
        }
    }

    pub(super) fn process_ship_sales(&mut self) -> Vec<ShipSaleInfo> {
        let mut sales_updates: Vec<(EntityId, EntityId, EntityId)> = self
            .civilian_ai
            .iter()
            .filter_map(|(id, ai)| match ai.state {
                CivilianShipState::WaitingToUnload { route, .. }
                    if self
                        .cargo
                        .get(id)
                        .is_some_and(|cargo| cargo.current_load > 0.0) =>
                {
                    Some((*id, ai.home_base, route.sell_body))
                }
                _ => None,
            })
            .collect();
        sales_updates.sort_by_key(|(ship_id, _, _)| *ship_id);

        let mut sales_info: Vec<ShipSaleInfo> = Vec::new();
        let mut remaining_throughput: std::collections::HashMap<EntityId, f32> =
            std::collections::HashMap::new();
        let mut used_berths: std::collections::HashMap<EntityId, u32> =
            std::collections::HashMap::new();
        let layer = crate::world::types::ConstructionLayer::Orbit;

        for (ship_id, home_base_id, destination_id) in sales_updates {
            let (dock_throughput, berth_capacity) = self
                .orbital_dock_capacity(destination_id)
                .unwrap_or((0.0, 0));
            let throughput = *remaining_throughput
                .entry(destination_id)
                .or_insert(dock_throughput);
            let berths = used_berths.entry(destination_id).or_insert(0);
            if throughput <= 0.0 || *berths >= berth_capacity {
                continue;
            }
            *berths += 1;

            let mut cargo_contents: Vec<_> = self.cargo[&ship_id]
                .contents
                .iter()
                .map(|(resource, amount)| (*resource, *amount))
                .collect();
            cargo_contents.sort_by_key(|(resource, _)| *resource);
            let mut accepted_cargo = Vec::new();
            let mut total_value = 0.0;

            for (resource, onboard) in cargo_contents {
                let procurement_key = crate::world::types::ProcurementKey { layer, resource };
                let Some(quote) = self.procurement_quote(destination_id, procurement_key) else {
                    continue;
                };
                let remaining = remaining_throughput[&destination_id];
                let requested = onboard.min(quote.wanted_quantity).min(remaining);
                if requested <= 0.0 {
                    break;
                }
                let storage_capacity = self
                    .infrastructure
                    .get(&destination_id)
                    .map(|infrastructure| infrastructure.accepting_storage_capacity(layer))
                    .unwrap_or(0.0);
                let accepted = self
                    .celestial_data
                    .get_mut(&destination_id)
                    .map(|body| {
                        body.deposit_bounded_at(layer, resource, requested, storage_capacity)
                    })
                    .unwrap_or(0.0);
                if accepted == 0.0 {
                    continue;
                }
                let value = accepted as f64 * quote.unit_price;
                let buyer = self.procurement_account(destination_id);
                let seller = crate::world::types::EconomicAccount::Civilian(home_base_id);
                if !self.transfer_credits(buyer, seller, value) {
                    self.celestial_data
                        .get_mut(&destination_id)
                        .unwrap()
                        .withdraw_at(layer, resource, accepted);
                    continue;
                }

                self.cargo
                    .get_mut(&ship_id)
                    .unwrap()
                    .remove(resource, accepted);
                *self
                    .celestial_data
                    .get_mut(&destination_id)
                    .unwrap()
                    .procurement_spend
                    .entry(procurement_key)
                    .or_insert(0.0) += value;
                *remaining_throughput.get_mut(&destination_id).unwrap() -= accepted;
                total_value += value;
                accepted_cargo.push((resource, accepted));
            }

            if accepted_cargo.is_empty() {
                *used_berths.get_mut(&destination_id).unwrap() -= 1;
                continue;
            }
            if self.cargo[&ship_id].current_load == 0.0 {
                self.civilian_ai.get_mut(&ship_id).unwrap().state = CivilianShipState::Idle;
            }
            sales_info.push((ship_id, total_value, destination_id, accepted_cargo));
        }

        sales_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::EntityInfrastructure;
    use crate::location::Point;
    use crate::world::types::{
        CelestialBodyData, ConstructionLayer, InfrastructureType, ProcurementKey,
        ProcurementPolicy, RawResource,
    };

    fn metals_route(target_body: EntityId, sell_body: EntityId) -> MiningRoute {
        MiningRoute {
            target_body,
            resource: RawResource::Metals,
            sell_body,
        }
    }

    fn mining_market_world() -> (World, EntityId, EntityId, EntityId, EntityId) {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let home_id = world.spawn_planet("home".to_string(), star_id, 6.0, 0.0, 0.0);
        let first_mine = world.spawn_planet("first mine".to_string(), star_id, 10.0, 0.0, 0.0);
        let second_mine = world.spawn_planet("second mine".to_string(), star_id, 10.0, 0.0, 0.0);
        let buyer_id = world.spawn_planet("buyer".to_string(), star_id, 16.0, 0.0, 0.0);
        for body_id in [home_id, first_mine, second_mine, buyer_id] {
            world
                .celestial_data
                .get_mut(&body_id)
                .unwrap()
                .yields
                .clear();
        }
        world
            .celestial_data
            .get_mut(&first_mine)
            .unwrap()
            .yields
            .insert(RawResource::Metals, 1.0);
        world
            .celestial_data
            .get_mut(&second_mine)
            .unwrap()
            .yields
            .insert(RawResource::Metals, 4.0);
        world.set_player_controlled(buyer_id);
        world.player_credits = 10_000.0;
        let infrastructure = world.infrastructure.get_mut(&buyer_id).unwrap();
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDepot, 1);
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDock, 1);
        world
            .celestial_data
            .get_mut(&buyer_id)
            .unwrap()
            .procurement_policies
            .insert(
                ProcurementKey {
                    layer: ConstructionLayer::Orbit,
                    resource: Storable::Raw(RawResource::Metals),
                },
                ProcurementPolicy {
                    enabled: true,
                    reserve_target: 80.0,
                    maximum_unit_price: 10.0,
                    periodic_spend_cap: None,
                },
            );
        let ship_id = world.spawn_mining_ship("miner".to_string(), Point { x: 6, y: 0 }, home_id);
        (world, ship_id, first_mine, second_mine, buyer_id)
    }

    #[test]
    fn route_estimate_accounts_for_live_quantity_yield_time_and_costs() {
        let (world, ship_id, _, mine_id, buyer_id) = mining_market_world();

        let estimate = world
            .estimate_mining_route(ship_id, metals_route(mine_id, buyer_id))
            .unwrap();

        assert_eq!(estimate.sale_quantity, 80.0);
        assert_eq!(estimate.mining_yield, 4.0);
        assert_eq!(estimate.unit_price, 4.0);
        assert_eq!(estimate.sale_revenue, 320.0);
        assert_eq!(estimate.mining_time, 20.0);
        assert!(estimate.travel_distance > 0.0);
        assert_eq!(estimate.travel_time, estimate.travel_distance / 2.0);
        assert_eq!(
            estimate.operating_cost,
            estimate.travel_distance * world.civilian_economy_config.travel_cost_per_distance
                + estimate.mining_time * world.civilian_economy_config.mining_cost_per_second
                + world.civilian_economy_config.ship_maintenance_per_cycle
                + world.civilian_economy_config.docking_fee
        );
        assert_eq!(
            estimate.expected_profit,
            estimate.sale_revenue - estimate.operating_cost
        );
    }

    #[test]
    fn autonomous_route_prefers_profit_per_time_and_ignores_losses() {
        let (mut world, ship_id, first_mine, second_mine, buyer_id) = mining_market_world();

        let best = world.best_mining_opportunity(ship_id).unwrap();
        assert_eq!(best.route, metals_route(second_mine, buyer_id));

        world
            .celestial_data
            .get_mut(&second_mine)
            .unwrap()
            .yields
            .insert(RawResource::Metals, 1.0);
        let first = world
            .estimate_mining_route(ship_id, metals_route(first_mine, buyer_id))
            .unwrap();
        let second = world
            .estimate_mining_route(ship_id, metals_route(second_mine, buyer_id))
            .unwrap();
        assert_eq!(first.profit_per_second, second.profit_per_second);
        assert_eq!(
            world.best_mining_opportunity(ship_id).unwrap().route,
            metals_route(first_mine, buyer_id)
        );

        world.civilian_economy_config.ship_maintenance_per_cycle = 10_000.0;
        assert!(world.best_mining_opportunity(ship_id).is_none());
    }

    #[test]
    fn manual_routes_run_even_when_no_autonomous_route_is_profitable() {
        let (mut world, ship_id, first_mine, _, buyer_id) = mining_market_world();
        let manual_route = metals_route(first_mine, buyer_id);
        world.civilian_economy_config.ship_maintenance_per_cycle = 10_000.0;
        world.set_mining_route(ship_id, Some(manual_route));

        world.update_civilian_ships(0.0);

        assert_eq!(
            world.civilian_ai[&ship_id].state,
            CivilianShipState::MovingToMine {
                route: manual_route,
                cargo_target: 80.0,
            }
        );
    }

    #[test]
    fn autonomous_ship_rechecks_the_market_after_unloading() {
        let (mut world, ship_id, first_mine, second_mine, buyer_id) = mining_market_world();
        let completed_route = metals_route(first_mine, buyer_id);
        world
            .cargo
            .get_mut(&ship_id)
            .unwrap()
            .add(Storable::Raw(RawResource::Metals), 1.0);
        world.civilian_ai.get_mut(&ship_id).unwrap().state = CivilianShipState::WaitingToUnload {
            route: completed_route,
            cargo_target: 1.0,
        };
        world.enable_civilian_ai = true;

        world.process_ship_sales();
        assert_eq!(world.civilian_ai[&ship_id].state, CivilianShipState::Idle);
        world.update_civilian_ships(0.0);

        assert_eq!(
            world.civilian_ai[&ship_id].state,
            CivilianShipState::MovingToMine {
                route: metals_route(second_mine, buyer_id),
                cargo_target: 79.0,
            }
        );
    }

    #[test]
    fn autonomous_trip_mines_only_the_quoted_quantity() {
        let (mut world, ship_id, _, mine_id, buyer_id) = mining_market_world();
        let route = metals_route(mine_id, buyer_id);
        world.civilian_ai.get_mut(&ship_id).unwrap().state = CivilianShipState::Mining {
            route,
            cargo_target: 80.0,
            mine_time: 0,
        };

        world.process_ship_mining(100.0);

        assert_eq!(world.cargo[&ship_id].current_load, 80.0);
        assert_eq!(
            world.cargo[&ship_id].contents[&Storable::Raw(RawResource::Metals)],
            80.0
        );
    }

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
        let home_id = world.spawn_planet("home".to_string(), star_id, 8.0, 0.0, 0.0);
        let planet_id = world.spawn_planet("earth".to_string(), star_id, 10.0, 0.0, 0.0);
        world.set_player_controlled(planet_id);
        world.player_credits = 100.0;
        world
            .infrastructure
            .get_mut(&planet_id)
            .unwrap()
            .infra
            .insert(InfrastructureType::OrbitalDepot, 1);
        world
            .infrastructure
            .get_mut(&planet_id)
            .unwrap()
            .infra
            .insert(InfrastructureType::OrbitalDock, 1);
        let key = ProcurementKey {
            layer: ConstructionLayer::Orbit,
            resource: Storable::Raw(RawResource::Metals),
        };
        let body = world.celestial_data.get_mut(&planet_id).unwrap();
        body.deposit_bounded_at(
            ConstructionLayer::Orbit,
            Storable::Good(crate::world::types::Good::Food),
            995.0,
            1_000.0,
        );
        body.procurement_policies.insert(
            key,
            ProcurementPolicy {
                enabled: true,
                reserve_target: 10.0,
                maximum_unit_price: 10.0,
                periodic_spend_cap: None,
            },
        );
        let ship_id = world.spawn_mining_ship("miner".to_string(), Point { x: 0, y: 0 }, home_id);
        world
            .cargo
            .get_mut(&ship_id)
            .unwrap()
            .add(Storable::Raw(RawResource::Metals), 10.0);
        world.civilian_ai.get_mut(&ship_id).unwrap().state = CivilianShipState::WaitingToUnload {
            route: metals_route(home_id, planet_id),
            cargo_target: 10.0,
        };

        let sales = world.process_ship_sales();

        assert_eq!(sales.len(), 1);
        assert_eq!(sales[0].3, vec![(Storable::Raw(RawResource::Metals), 5.0)]);
        assert_eq!(world.cargo[&ship_id].current_load, 5.0);
        assert_eq!(world.player_credits, 80.0);
        assert_eq!(world.celestial_data[&home_id].credits, 20.0);
        assert_eq!(world.celestial_data[&planet_id].credits, 0.0);
        assert_eq!(
            world.civilian_ai[&ship_id].state,
            CivilianShipState::WaitingToUnload {
                route: metals_route(home_id, planet_id),
                cargo_target: 10.0,
            }
        );
        assert_eq!(
            world.celestial_data[&planet_id].stored_units_at(ConstructionLayer::Orbit),
            1_000.0
        );
    }

    #[test]
    fn mining_route_sell_destination_does_not_replace_home_economy() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let home_id = world.spawn_planet("home".to_string(), star_id, 8.0, 0.0, 0.0);
        let mine_id = world.spawn_planet("mine".to_string(), star_id, 10.0, 0.0, 0.0);
        let sell_id = world.spawn_planet("market".to_string(), star_id, 12.0, 0.0, 0.0);
        let ship_id = world.spawn_mining_ship("miner".to_string(), Point { x: 0, y: 0 }, home_id);

        world.set_mining_route(
            ship_id,
            Some(MiningRoute {
                target_body: mine_id,
                resource: RawResource::Metals,
                sell_body: sell_id,
            }),
        );

        assert_eq!(world.civilian_ai[&ship_id].home_base, home_id);
        assert_eq!(world.mining_routes[&ship_id].sell_body, sell_id);
    }

    #[test]
    fn delivery_is_limited_by_remaining_dock_throughput() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let home_id = world.spawn_planet("home".to_string(), star_id, 8.0, 0.0, 0.0);
        let destination_id = world.spawn_planet("market".to_string(), star_id, 10.0, 0.0, 0.0);
        world.set_player_controlled(destination_id);
        world.player_credits = 10_000.0;
        let infrastructure = world.infrastructure.get_mut(&destination_id).unwrap();
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDepot, 1);
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDock, 1);
        let procurement_key = ProcurementKey {
            layer: ConstructionLayer::Orbit,
            resource: Storable::Raw(RawResource::Metals),
        };
        world
            .celestial_data
            .get_mut(&destination_id)
            .unwrap()
            .procurement_policies
            .insert(
                procurement_key,
                ProcurementPolicy {
                    enabled: true,
                    reserve_target: 200.0,
                    maximum_unit_price: 100.0,
                    periodic_spend_cap: Some(400.0),
                },
            );
        let ship_id =
            world.spawn_mining_ship("large miner".to_string(), Point { x: 0, y: 0 }, home_id);
        world.cargo.get_mut(&ship_id).unwrap().capacity = 200.0;
        world
            .cargo
            .get_mut(&ship_id)
            .unwrap()
            .add(Storable::Raw(RawResource::Metals), 150.0);
        world.civilian_ai.get_mut(&ship_id).unwrap().state = CivilianShipState::WaitingToUnload {
            route: metals_route(home_id, destination_id),
            cargo_target: 150.0,
        };

        let sales = world.process_ship_sales();

        assert_eq!(
            sales[0].3,
            vec![(Storable::Raw(RawResource::Metals), 100.0)]
        );
        assert_eq!(world.cargo[&ship_id].current_load, 50.0);
        assert_eq!(
            world.celestial_data[&destination_id]
                .amount_at(ConstructionLayer::Orbit, Storable::Raw(RawResource::Metals)),
            100.0
        );
        assert_eq!(
            world.celestial_data[&destination_id].procurement_spend[&procurement_key],
            400.0
        );
        assert!(world.process_ship_sales().is_empty());
        assert_eq!(world.cargo[&ship_id].current_load, 50.0);
    }

    #[test]
    fn simultaneous_arrivals_use_ship_id_order_and_available_berths() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let first_home = world.spawn_planet("first home".to_string(), star_id, 8.0, 0.0, 0.0);
        let second_home = world.spawn_planet("second home".to_string(), star_id, 9.0, 0.0, 0.0);
        let destination_id = world.spawn_planet("market".to_string(), star_id, 10.0, 0.0, 0.0);
        world.set_player_controlled(destination_id);
        world.player_credits = 10_000.0;
        let infrastructure = world.infrastructure.get_mut(&destination_id).unwrap();
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDepot, 1);
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDock, 1);
        world
            .celestial_data
            .get_mut(&destination_id)
            .unwrap()
            .procurement_policies
            .insert(
                ProcurementKey {
                    layer: ConstructionLayer::Orbit,
                    resource: Storable::Raw(RawResource::Metals),
                },
                ProcurementPolicy {
                    enabled: true,
                    reserve_target: 200.0,
                    maximum_unit_price: 100.0,
                    periodic_spend_cap: None,
                },
            );
        let first_ship =
            world.spawn_mining_ship("first".to_string(), Point { x: 0, y: 0 }, first_home);
        let second_ship =
            world.spawn_mining_ship("second".to_string(), Point { x: 0, y: 0 }, second_home);
        for ship_id in [first_ship, second_ship] {
            world
                .cargo
                .get_mut(&ship_id)
                .unwrap()
                .add(Storable::Raw(RawResource::Metals), 60.0);
            world.civilian_ai.get_mut(&ship_id).unwrap().state =
                CivilianShipState::WaitingToUnload {
                    route: metals_route(first_home, destination_id),
                    cargo_target: 60.0,
                };
        }
        assert_eq!(world.ships_waiting_to_unload(destination_id), 2);

        let sales = world.process_ship_sales();

        assert_eq!(sales.len(), 1);
        assert_eq!(sales[0].0, first_ship);
        assert_eq!(world.cargo[&first_ship].current_load, 0.0);
        assert_eq!(world.cargo[&second_ship].current_load, 60.0);
        assert_eq!(world.ships_waiting_to_unload(destination_id), 1);
        assert_eq!(
            world.civilian_ai[&first_ship].state,
            CivilianShipState::Idle
        );
        assert_eq!(
            world.civilian_ai[&second_ship].state,
            CivilianShipState::WaitingToUnload {
                route: metals_route(first_home, destination_id),
                cargo_target: 60.0,
            }
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
            crate::world::resources::get_local_price(
                &world,
                planet_id,
                Storable::Raw(crate::world::types::RawResource::Volatiles)
            ) > crate::world::resources::get_resource_base_price(Storable::Raw(
                crate::world::types::RawResource::Volatiles
            ))
        );
    }
}
