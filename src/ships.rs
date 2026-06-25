use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::world::types::{RawResource, Storable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipType {
    Frigate,
    MiningShip,
}

impl ShipType {
    /// build cost, deducted from the shipyard body's stocks (like buildings).
    /// placeholder values — tune to taste.
    pub fn build_cost(self) -> HashMap<Storable, f32> {
        let mut costs = HashMap::new();
        match self {
            ShipType::Frigate => {
                costs.insert(Storable::Raw(RawResource::Metals), 80.0);
                costs.insert(Storable::Raw(RawResource::Crystals), 30.0);
            }
            ShipType::MiningShip => {
                costs.insert(Storable::Raw(RawResource::Metals), 50.0);
                costs.insert(Storable::Raw(RawResource::Crystals), 15.0);
            }
        }
        costs
    }
}
