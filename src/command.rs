use crate::location::PointF64;
use crate::ships::ShipType;
use crate::world::types::InfrastructureType;
use crate::world::EntityId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    MoveShip {
        ship_id: EntityId,
        destination: PointF64,
    },
    Build {
        entity_id: EntityId,
        infrastructure_type: InfrastructureType,
        amount: u32,
    },
    BuildShip {
        shipyard_entity_id: EntityId,
        ship_type: ShipType,
        civilian_credit_cost: Option<f64>,
    },
}
