use std::collections::HashMap;

pub type EntityId = u32;

pub enum EntityType {
    Planet,
    Space,
}

pub type EntityTypeMap = HashMap<EntityId, EntityType>;
