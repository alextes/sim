use std::collections::HashMap;

pub type EntityId = u32;

pub enum EntityType {
    Moon,
    Planet,
    Space,
    Star,
}

pub type EntityTypeMap = HashMap<EntityId, EntityType>;
