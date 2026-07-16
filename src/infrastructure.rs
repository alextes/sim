#![allow(dead_code)] // TODO remove later

use crate::world::types::{InfrastructureType, RawResource, Storable};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// represents the infrastructure on an entity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityInfrastructure {
    /// a map from infrastructure type to the number of completed units.
    pub infra: HashMap<InfrastructureType, u32>,
    /// infrastructure units waiting to be constructed.
    pub build_queue: VecDeque<(InfrastructureType, u32)>,
    /// progress on the current construction item.
    pub construction_progress: f32,
    /// the name of the entity that owns this infrastructure.
    pub entity_name: String,
}

impl EntityInfrastructure {
    /// creates a new, empty set of infrastructure.
    pub fn new(entity_name: &str) -> Self {
        Self {
            infra: HashMap::new(),
            build_queue: VecDeque::new(),
            construction_progress: 0.0,
            entity_name: entity_name.to_string(),
        }
    }

    /// queues a number of units of a given infrastructure type to be built.
    pub fn queue_build(&mut self, infrastructure: InfrastructureType, count: u32) {
        self.build_queue.push_back((infrastructure, count));
    }

    /// gets the completed count of a specific infrastructure type.
    pub fn get_count(&self, infrastructure: InfrastructureType) -> u32 {
        self.infra.get(&infrastructure).copied().unwrap_or(0)
    }

    /// gets the queued count of a specific infrastructure type.
    pub fn get_queued_count(&self, infrastructure: InfrastructureType) -> u32 {
        self.build_queue
            .iter()
            .filter(|(queued, _)| *queued == infrastructure)
            .map(|(_, count)| *count)
            .sum()
    }

    /// processes the construction queue for a given time step.
    pub fn process_construction(&mut self, dt: f32) {
        if self.build_queue.is_empty() {
            return;
        }

        let construction_rate = self.get_count(InfrastructureType::ConstructionFactory) as f32;
        if construction_rate == 0.0 {
            return; // no construction capacity
        }

        self.construction_progress += construction_rate * dt;

        while self.construction_progress >= 1.0 {
            self.construction_progress -= 1.0;

            if let Some((infrastructure, count)) = self.build_queue.front_mut() {
                *self.infra.entry(*infrastructure).or_insert(0) += 1;
                *count -= 1;
                tracing::debug!(
                    "entity {} finished constructing 1 unit of {:?}, {} remaining in queue",
                    self.entity_name,
                    *infrastructure,
                    *count
                );
            }

            if let Some((_, count)) = self.build_queue.front() {
                if *count == 0 {
                    self.build_queue.pop_front();
                }
            } else {
                break; // queue is empty
            }
        }
    }

    /// returns the total cost to build a number of units of an infrastructure type.
    pub fn get_build_costs(
        infrastructure_type: InfrastructureType,
        count: u32,
    ) -> HashMap<Storable, f32> {
        let mut costs = HashMap::new();
        let base_costs = EntityInfrastructure::get_build_cost(infrastructure_type);
        for (resource, cost) in base_costs {
            costs.insert(resource, cost * count as f32);
        }
        costs
    }

    /// returns the base cost for a single unit of an infrastructure type.
    pub fn get_build_cost(infrastructure_type: InfrastructureType) -> HashMap<Storable, f32> {
        let mut costs = HashMap::new();
        match infrastructure_type {
            InfrastructureType::Spaceport => {
                costs.insert(Storable::Raw(RawResource::Metals), 100.0);
            }
            InfrastructureType::SolarPanel => {
                costs.insert(Storable::Raw(RawResource::Metals), 10.0);
                costs.insert(Storable::Raw(RawResource::Crystals), 20.0);
            }
            InfrastructureType::Mine => {
                costs.insert(Storable::Raw(RawResource::Metals), 50.0);
            }
            InfrastructureType::Shipyard => {
                costs.insert(Storable::Raw(RawResource::Metals), 200.0);
            }
            InfrastructureType::FuelCellCracker => {
                costs.insert(Storable::Raw(RawResource::Metals), 100.0);
                costs.insert(Storable::Raw(RawResource::Crystals), 75.0);
            }
            InfrastructureType::Farm => {
                costs.insert(Storable::Raw(RawResource::Metals), 20.0);
                costs.insert(Storable::Raw(RawResource::Organics), 50.0);
            }
            InfrastructureType::ConstructionFactory => {
                costs.insert(Storable::Raw(RawResource::Metals), 150.0);
            }
        }
        costs
    }

    /// returns a display name for an infrastructure type.
    pub fn infrastructure_name(infrastructure: InfrastructureType) -> &'static str {
        match infrastructure {
            InfrastructureType::Mine => "mine",
            InfrastructureType::FuelCellCracker => "fuel cell cracker",
            InfrastructureType::Farm => "farm",
            InfrastructureType::Shipyard => "shipyard",
            InfrastructureType::ConstructionFactory => "construction factory",
            InfrastructureType::Spaceport => "spaceport",
            InfrastructureType::SolarPanel => "orbital solar panel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::types::{InfrastructureType, RawResource, Storable};

    #[test]
    fn test_infrastructure_name() {
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::Mine),
            "mine"
        );
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::FuelCellCracker),
            "fuel cell cracker"
        );
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::Farm),
            "farm"
        );
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::Shipyard),
            "shipyard"
        );
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::ConstructionFactory),
            "construction factory"
        );
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::SolarPanel),
            "orbital solar panel"
        );
        assert_eq!(
            EntityInfrastructure::infrastructure_name(InfrastructureType::Spaceport),
            "spaceport"
        );
    }

    #[test]
    fn test_new_entity_infrastructure() {
        let infrastructure = EntityInfrastructure::new("test");
        assert!(infrastructure.infra.is_empty());
        assert!(infrastructure.build_queue.is_empty());
    }

    #[test]
    fn test_queue_and_get_count() {
        let mut infrastructure = EntityInfrastructure::new("test");
        assert_eq!(infrastructure.get_count(InfrastructureType::Mine), 0);

        infrastructure.queue_build(InfrastructureType::Mine, 1);
        assert_eq!(infrastructure.build_queue.len(), 1);

        infrastructure.queue_build(InfrastructureType::Mine, 3);
        assert_eq!(infrastructure.build_queue.len(), 2);

        assert_eq!(infrastructure.get_count(InfrastructureType::Farm), 0);
    }

    #[test]
    fn test_construction_queue() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure
            .infra
            .insert(InfrastructureType::ConstructionFactory, 1);

        infrastructure.queue_build(InfrastructureType::Mine, 2);
        assert_eq!(infrastructure.build_queue.len(), 1);

        infrastructure.process_construction(0.5);
        assert_eq!(infrastructure.get_count(InfrastructureType::Mine), 0);
        assert_eq!(infrastructure.build_queue.front().unwrap().1, 2);

        infrastructure.process_construction(0.5);
        assert_eq!(infrastructure.get_count(InfrastructureType::Mine), 1);
        assert_eq!(infrastructure.build_queue.front().unwrap().1, 1);

        infrastructure.process_construction(1.0);
        assert_eq!(infrastructure.get_count(InfrastructureType::Mine), 2);
        assert!(infrastructure.build_queue.is_empty());
    }

    #[test]
    fn test_get_build_cost() {
        let costs = EntityInfrastructure::get_build_cost(InfrastructureType::Mine);
        assert_eq!(costs.len(), 1);
        assert_eq!(costs.get(&Storable::Raw(RawResource::Metals)), Some(&50.0));

        let costs = EntityInfrastructure::get_build_cost(InfrastructureType::Farm);
        assert_eq!(costs.len(), 2);
        assert_eq!(costs.get(&Storable::Raw(RawResource::Metals)), Some(&20.0));
        assert_eq!(
            costs.get(&Storable::Raw(RawResource::Organics)),
            Some(&50.0)
        );

        let costs = EntityInfrastructure::get_build_cost(InfrastructureType::Spaceport);
        assert_eq!(costs.len(), 1);
        assert_eq!(costs.get(&Storable::Raw(RawResource::Metals)), Some(&100.0));
    }

    #[test]
    fn queued_count_sums_matching_queue_entries() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure.queue_build(InfrastructureType::Spaceport, 1);
        infrastructure.queue_build(InfrastructureType::SolarPanel, 4);
        infrastructure.queue_build(InfrastructureType::Spaceport, 2);

        assert_eq!(
            infrastructure.get_queued_count(InfrastructureType::Spaceport),
            3
        );
        assert_eq!(
            infrastructure.get_queued_count(InfrastructureType::SolarPanel),
            4
        );
    }
}
