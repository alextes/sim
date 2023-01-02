use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{entity::EntityId, Point, Viewport};

#[derive(Debug)]
pub struct LocationMap(HashMap<EntityId, Point>);

impl LocationMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_entity(&mut self, entity_id: EntityId, x: i32, y: i32) {
        self.0.insert(entity_id, Point { x, y });
    }

    pub fn translate_location(point: &Point, viewport: &Viewport) -> Point {
        Point {
            x: point.x - viewport.center.x,
            y: point.y - viewport.center.y,
        }
    }
}

impl Deref for LocationMap {
    type Target = HashMap<EntityId, Point>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocationMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
