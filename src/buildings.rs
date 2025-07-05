#![allow(dead_code)] // TODO remove later

use crate::world::types::{BuildingType, RawResource, Storable};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Represents the buildings on an entity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityBuildings {
    /// A map from building type to the number of units of that infrastructure.
    pub infra: HashMap<BuildingType, u32>,
    /// A queue of buildings to be constructed.
    pub build_queue: VecDeque<(BuildingType, u32)>,
    /// Progress on the current construction item.
    pub construction_progress: f32,
    /// The name of the entity that owns these buildings.
    pub entity_name: String,
}

impl EntityBuildings {
    /// Creates a new, empty set of buildings.
    pub fn new(entity_name: &str) -> Self {
        Self {
            infra: HashMap::new(),
            build_queue: VecDeque::new(),
            construction_progress: 0.0,
            entity_name: entity_name.to_string(),
        }
    }

    /// Queues a number of units of a given building/infra type to be built.
    pub fn queue_build(&mut self, building: BuildingType, count: u32) {
        self.build_queue.push_back((building, count));
    }

    /// Gets the count of a specific building/infra type.
    pub fn get_count(&self, building: BuildingType) -> u32 {
        self.infra.get(&building).copied().unwrap_or(0)
    }

    /// Processes the construction queue for a given time step.
    pub fn process_construction(&mut self, dt: f32) {
        if self.build_queue.is_empty() {
            return;
        }

        let construction_rate = self.get_count(BuildingType::ConstructionFactory) as f32;
        if construction_rate == 0.0 {
            return; // No construction capacity
        }

        self.construction_progress += construction_rate * dt;

        while self.construction_progress >= 1.0 {
            self.construction_progress -= 1.0;

            if let Some((building, count)) = self.build_queue.front_mut() {
                *self.infra.entry(*building).or_insert(0) += 1;
                *count -= 1;
                tracing::debug!(
                    "entity {} finished constructing 1 unit of {:?}, {} remaining in queue",
                    self.entity_name,
                    *building,
                    *count
                );
            }

            if let Some((_, count)) = self.build_queue.front() {
                if *count == 0 {
                    self.build_queue.pop_front();
                }
            } else {
                break; // Queue is empty
            }
        }
    }

    /// Returns the total cost to build a number of units of a building type.
    pub fn get_build_costs(b_type: BuildingType, count: u32) -> HashMap<Storable, f32> {
        let mut costs = HashMap::new();
        let base_costs = EntityBuildings::get_build_cost(b_type);
        for (resource, cost) in base_costs {
            costs.insert(resource, cost * count as f32);
        }
        costs
    }

    /// Returns the base cost for a single unit of a building type.
    pub fn get_build_cost(b_type: BuildingType) -> HashMap<Storable, f32> {
        let mut costs = HashMap::new();
        match b_type {
            BuildingType::SolarPanel => {
                costs.insert(Storable::Raw(RawResource::Metals), 10.0);
                costs.insert(Storable::Raw(RawResource::Crystals), 20.0);
            }
            BuildingType::Mine => {
                costs.insert(Storable::Raw(RawResource::Metals), 50.0);
            }
            BuildingType::Shipyard => {
                costs.insert(Storable::Raw(RawResource::Metals), 200.0);
            }
            BuildingType::FuelCellCracker => {
                costs.insert(Storable::Raw(RawResource::Metals), 100.0);
                costs.insert(Storable::Raw(RawResource::Crystals), 75.0);
            }
            BuildingType::Farm => {
                costs.insert(Storable::Raw(RawResource::Metals), 20.0);
                costs.insert(Storable::Raw(RawResource::Organics), 50.0);
            }
            BuildingType::ConstructionFactory => {
                costs.insert(Storable::Raw(RawResource::Metals), 150.0);
            }
        }
        costs
    }

    /// Helper to get a display name for a building type.
    pub fn building_name(building: BuildingType) -> &'static str {
        match building {
            BuildingType::Mine => "mine",
            BuildingType::FuelCellCracker => "fuel cell cracker",
            BuildingType::Farm => "farm",
            BuildingType::Shipyard => "shipyard",
            BuildingType::ConstructionFactory => "construction factory",
            BuildingType::SolarPanel => "solar panel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::types::{BuildingType, RawResource, Storable};

    #[test]
    fn test_building_name() {
        assert_eq!(EntityBuildings::building_name(BuildingType::Mine), "mine");
        assert_eq!(
            EntityBuildings::building_name(BuildingType::FuelCellCracker),
            "fuel cell cracker"
        );
        assert_eq!(EntityBuildings::building_name(BuildingType::Farm), "farm");
        assert_eq!(
            EntityBuildings::building_name(BuildingType::Shipyard),
            "shipyard"
        );
        assert_eq!(
            EntityBuildings::building_name(BuildingType::ConstructionFactory),
            "construction factory"
        );
        assert_eq!(
            EntityBuildings::building_name(BuildingType::SolarPanel),
            "solar panel"
        );
    }

    #[test]
    fn test_new_entity_buildings() {
        let eb = EntityBuildings::new("test");
        assert!(eb.infra.is_empty());
        assert!(eb.build_queue.is_empty());
    }

    #[test]
    fn test_queue_and_get_count() {
        let mut buildings = EntityBuildings::new("test");
        assert_eq!(buildings.get_count(BuildingType::Mine), 0);

        buildings.queue_build(BuildingType::Mine, 1);
        assert_eq!(buildings.build_queue.len(), 1);

        buildings.queue_build(BuildingType::Mine, 3);
        assert_eq!(buildings.build_queue.len(), 2);

        assert_eq!(buildings.get_count(BuildingType::Farm), 0);
    }

    #[test]
    fn test_construction_queue() {
        let mut buildings = EntityBuildings::new("test");
        buildings.infra.insert(BuildingType::ConstructionFactory, 1);

        buildings.queue_build(BuildingType::Mine, 2);
        assert_eq!(buildings.build_queue.len(), 1);

        // Process construction for 0.5s, not enough to build one unit
        buildings.process_construction(0.5);
        assert_eq!(buildings.get_count(BuildingType::Mine), 0);
        assert_eq!(buildings.build_queue.front().unwrap().1, 2);

        // Process construction for another 0.5s, enough to build one unit
        buildings.process_construction(0.5);
        assert_eq!(buildings.get_count(BuildingType::Mine), 1);
        assert_eq!(buildings.build_queue.front().unwrap().1, 1);

        // Process construction for 1.0s, enough to build the second unit
        buildings.process_construction(1.0);
        assert_eq!(buildings.get_count(BuildingType::Mine), 2);
        assert!(buildings.build_queue.is_empty());
    }

    #[test]
    fn test_get_build_cost() {
        let costs = EntityBuildings::get_build_cost(BuildingType::Mine);
        assert_eq!(costs.len(), 1);
        assert_eq!(costs.get(&Storable::Raw(RawResource::Metals)), Some(&50.0));

        let costs = EntityBuildings::get_build_cost(BuildingType::Farm);
        assert_eq!(costs.len(), 2);
        assert_eq!(costs.get(&Storable::Raw(RawResource::Metals)), Some(&20.0));
        assert_eq!(
            costs.get(&Storable::Raw(RawResource::Organics)),
            Some(&50.0)
        );
    }
}
