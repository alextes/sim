use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::world::types::{RawResource, Storable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipType {
    Frigate,
    MiningShip,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipBuildCost {
    pub resource: Storable,
    pub quantity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipBuildable {
    pub ship_type: ShipType,
    pub name: &'static str,
    pub costs: &'static [ShipBuildCost],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipBuildShortfall {
    pub resource: Storable,
    pub required: f32,
    pub available: f32,
}

const FRIGATE_BUILD_COSTS: &[ShipBuildCost] = &[
    ShipBuildCost {
        resource: Storable::Raw(RawResource::Metals),
        quantity: 80.0,
    },
    ShipBuildCost {
        resource: Storable::Raw(RawResource::Crystals),
        quantity: 30.0,
    },
];

const MINING_SHIP_BUILD_COSTS: &[ShipBuildCost] = &[
    ShipBuildCost {
        resource: Storable::Raw(RawResource::Metals),
        quantity: 50.0,
    },
    ShipBuildCost {
        resource: Storable::Raw(RawResource::Crystals),
        quantity: 15.0,
    },
];

const BUILDABLE_SHIPS: &[ShipBuildable] = &[
    ShipBuildable {
        ship_type: ShipType::Frigate,
        name: "frigate",
        costs: FRIGATE_BUILD_COSTS,
    },
    ShipBuildable {
        ship_type: ShipType::MiningShip,
        name: "mining ship",
        costs: MINING_SHIP_BUILD_COSTS,
    },
];

pub fn buildable_ships() -> &'static [ShipBuildable] {
    BUILDABLE_SHIPS
}

pub fn buildable_ship(ship_type: ShipType) -> Option<&'static ShipBuildable> {
    BUILDABLE_SHIPS
        .iter()
        .find(|buildable| buildable.ship_type == ship_type)
}

impl ShipBuildable {
    pub fn first_shortfall(self, stocks: &HashMap<Storable, f32>) -> Option<ShipBuildShortfall> {
        self.costs.iter().find_map(|cost| {
            let available = stocks.get(&cost.resource).copied().unwrap_or(0.0);
            (available < cost.quantity).then_some(ShipBuildShortfall {
                resource: cost.resource,
                required: cost.quantity,
                available,
            })
        })
    }

    pub fn can_afford(self, stocks: &HashMap<Storable, f32>) -> bool {
        self.first_shortfall(stocks).is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buildable_ships_list_known_types_with_ordered_costs() {
        let buildables = buildable_ships();

        assert_eq!(buildables.len(), 2);
        assert_eq!(buildables[0].ship_type, ShipType::Frigate);
        assert_eq!(buildables[1].ship_type, ShipType::MiningShip);
        assert_eq!(
            buildables[0].costs,
            [
                ShipBuildCost {
                    resource: Storable::Raw(RawResource::Metals),
                    quantity: 80.0,
                },
                ShipBuildCost {
                    resource: Storable::Raw(RawResource::Crystals),
                    quantity: 30.0,
                },
            ]
        );
    }

    #[test]
    fn first_shortfall_uses_definition_order() {
        let buildable = buildable_ship(ShipType::Frigate).expect("frigate is buildable");
        let stocks = HashMap::new();

        assert_eq!(
            buildable.first_shortfall(&stocks),
            Some(ShipBuildShortfall {
                resource: Storable::Raw(RawResource::Metals),
                required: 80.0,
                available: 0.0,
            })
        );
    }
}
