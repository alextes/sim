use crate::infrastructure::InfrastructureCategory;
use crate::ships::{buildable_ship, ShipBuildShortfall, ShipType};
use crate::world::types::ConstructionLayer;
use crate::world::{EntityId, World};

/// layer whose stocks pay for ship construction.
pub const SHIP_BUILD_LAYER: ConstructionLayer = ConstructionLayer::Surface;

/// first reason a body cannot build a ship right now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShipBuildBlocker {
    /// the ship type has no build definition.
    UnavailableShipType,
    /// the body has no economy data to pay from.
    MissingStocks,
    /// the body has no operational shipbuilding infrastructure.
    NoOperationalShipyard,
    /// the body lacks a construction material.
    Materials(ShipBuildShortfall),
    /// the paying civilian economy lacks credits.
    Credits { required: f64, available: f64 },
}

impl std::fmt::Display for ShipBuildBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnavailableShipType => write!(f, "ship type is not buildable"),
            Self::MissingStocks => write!(f, "missing shipyard stocks"),
            Self::NoOperationalShipyard => write!(f, "no operational shipyard"),
            Self::Materials(shortfall) => write!(
                f,
                "not enough {:?} (needs {}, has {})",
                shortfall.resource, shortfall.required, shortfall.available
            ),
            Self::Credits {
                required,
                available,
            } => write!(f, "not enough credits (needs {required}, has {available})"),
        }
    }
}

impl World {
    /// checks whether a body can build a ship now: an operational shipyard,
    /// construction materials in the build layer, and, for civilian builds,
    /// enough civilian credits. returns the first blocker in that order.
    pub fn ship_build_blocker(
        &self,
        shipyard_id: EntityId,
        ship_type: ShipType,
        civilian_credit_cost: Option<f64>,
    ) -> Option<ShipBuildBlocker> {
        let Some(buildable) = buildable_ship(ship_type) else {
            return Some(ShipBuildBlocker::UnavailableShipType);
        };
        let Some(body) = self.celestial_data.get(&shipyard_id) else {
            return Some(ShipBuildBlocker::MissingStocks);
        };
        let has_shipyard = self.infrastructure.get(&shipyard_id).is_some_and(|infra| {
            infra.operational_units_in_category(InfrastructureCategory::Shipbuilding) > 0
        });
        if !has_shipyard {
            return Some(ShipBuildBlocker::NoOperationalShipyard);
        }
        if let Some(shortfall) = buildable.first_shortfall(body.stocks_at(SHIP_BUILD_LAYER)) {
            return Some(ShipBuildBlocker::Materials(shortfall));
        }
        if let Some(required) = civilian_credit_cost {
            if body.credits < required {
                return Some(ShipBuildBlocker::Credits {
                    required,
                    available: body.credits,
                });
            }
        }
        None
    }
}
