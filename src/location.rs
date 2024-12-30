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

    // We render the viewport, the anchor is the top left corner of the viewport. So we need to
    // subtract the anchor from the universe coordinate to get the viewport coordinate.
    pub fn translate_location(uni_coord: &Point, viewport: &Viewport) -> Point {
        Point {
            x: uni_coord.x - viewport.anchor.x,
            y: uni_coord.y - viewport.anchor.y,
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
