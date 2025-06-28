use crate::world::types::Storable;
use crate::world::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    /// clears all cargo.
    pub fn clear(&mut self) {
        self.current_load = 0.0;
        self.contents.clear();
    }
}

/// The state machine for civilian ships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivilianShipAI {
    pub state: CivilianShipState,
    pub home_base: EntityId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CivilianShipState {
    Idle,
    MovingToMine { target: EntityId },
    Mining { target: EntityId, mine_time: f64 },
    ReturningToSell,
}
