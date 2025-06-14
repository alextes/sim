use crate::buildings::BuildingType;
use crate::location::PointF64;
use crate::ships::ShipType;
use crate::world::EntityId;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    MoveShip {
        ship_id: EntityId,
        destination: PointF64,
    },
    BuildShip {
        shipyard_entity_id: EntityId,
        ship_type: ShipType,
    },
    BuildBuilding {
        entity_id: EntityId,
        building_type: BuildingType,
    },
}
