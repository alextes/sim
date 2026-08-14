use crate::world::types::RawResource;
use crate::world::types::Storable;
use crate::world::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiningRoute {
    pub target_body: EntityId,
    pub resource: RawResource,
    pub sell_body: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningRouteEstimate {
    pub route: MiningRoute,
    pub sale_quantity: f32,
    pub mining_yield: f32,
    pub unit_price: f64,
    pub sale_revenue: f64,
    pub travel_distance: f64,
    pub travel_time: f64,
    pub mining_time: f64,
    pub operating_cost: f64,
    pub cycle_time: f64,
    pub expected_profit: f64,
    pub profit_per_second: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CivilianEconomyConfig {
    pub mining_units_per_yield_second: f32,
    pub travel_cost_per_distance: f64,
    pub mining_cost_per_second: f64,
    pub ship_maintenance_per_cycle: f64,
    pub docking_fee: f64,
}

impl Default for CivilianEconomyConfig {
    fn default() -> Self {
        Self {
            mining_units_per_yield_second: 1.0,
            travel_cost_per_distance: 0.25,
            mining_cost_per_second: 0.1,
            ship_maintenance_per_cycle: 10.0,
            docking_fee: 5.0,
        }
    }
}

/// Holds resources for an entity, like a ship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cargo {
    pub capacity: f32,
    pub current_load: f32,
    pub contents: HashMap<Storable, f32>,
}

impl Cargo {
    pub fn new(capacity: f32) -> Self {
        Self {
            capacity,
            current_load: 0.0,
            contents: HashMap::new(),
        }
    }

    /// adds resources to the cargo, returning the amount that couldn't be added.
    pub fn add(&mut self, resource: Storable, amount: f32) -> f32 {
        let space_available = self.capacity - self.current_load;
        let amount_to_add = amount.min(space_available);

        if amount_to_add > 0.0 {
            *self.contents.entry(resource).or_insert(0.0) += amount_to_add;
            self.current_load += amount_to_add;
        }

        amount - amount_to_add // returns leftover amount
    }

    /// removes up to the requested resource amount and returns what was removed.
    pub fn remove(&mut self, resource: Storable, amount: f32) -> f32 {
        let available = self.contents.get(&resource).copied().unwrap_or(0.0);
        let removed = amount.max(0.0).min(available);
        if removed > 0.0 {
            let empty = if let Some(stored) = self.contents.get_mut(&resource) {
                *stored -= removed;
                *stored == 0.0
            } else {
                false
            };
            if empty {
                self.contents.remove(&resource);
            }
            self.current_load -= removed;
        }
        removed
    }
}

/// The state machine for civilian ships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivilianShipAI {
    pub state: CivilianShipState,
    pub home_base: EntityId,
}

/// state for a civilian ship's ai.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CivilianShipState {
    Idle,
    MovingToMine {
        route: MiningRoute,
        cargo_target: f32,
    },
    Mining {
        route: MiningRoute,
        cargo_target: f32,
        mine_time: u64, // milliseconds spent mining during the current trip
    },
    ReturningToSell {
        route: MiningRoute,
        cargo_target: f32,
    },
    WaitingToUnload {
        route: MiningRoute,
        cargo_target: f32,
    },
}
