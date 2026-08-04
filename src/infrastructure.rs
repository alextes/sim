#![allow(dead_code)] // TODO remove later

use crate::world::types::{CelestialBodyData, EntityType, Good, InfrastructureType, Storable};
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

    /// processes the construction queue using capacity and material at the build layer.
    pub fn process_construction(
        &mut self,
        dt: f32,
        construction_capacity: f32,
        anchor_type: EntityType,
        body_data: &mut CelestialBodyData,
    ) {
        if self.build_queue.is_empty() {
            return;
        }

        let mut remaining_capacity = construction_capacity.max(0.0) * dt.max(0.0);
        if remaining_capacity == 0.0 {
            return;
        }

        while remaining_capacity > 0.0 {
            let Some(&(infrastructure_type, _)) = self.build_queue.front() else {
                break;
            };
            let Some(layer) = infrastructure_type.construction_layer(anchor_type) else {
                break;
            };
            let material_cost = Self::construction_material_cost(infrastructure_type);
            let material = body_data
                .stocks_at_mut(layer)
                .entry(Storable::Good(Good::ConstructionMaterials))
                .or_insert(0.0);
            let material_needed = (material_cost - self.construction_progress).max(0.0);
            let progress = remaining_capacity.min(*material).min(material_needed);

            if progress == 0.0 {
                break;
            }

            *material -= progress;
            remaining_capacity -= progress;
            self.construction_progress += progress;

            if self.construction_progress < material_cost {
                break;
            }

            self.construction_progress = 0.0;

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
                break;
            }
        }
    }

    /// returns construction material needed for one infrastructure unit.
    pub fn construction_material_cost(infrastructure_type: InfrastructureType) -> f32 {
        match infrastructure_type {
            InfrastructureType::Spaceport => 100.0,
            InfrastructureType::SolarPanel => 30.0,
            InfrastructureType::Mine => 50.0,
            InfrastructureType::Shipyard => 200.0,
            InfrastructureType::FuelCellCracker => 175.0,
            InfrastructureType::Farm => 70.0,
            InfrastructureType::ConstructionFactory => 150.0,
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
        HashMap::from([(
            Storable::Good(Good::ConstructionMaterials),
            Self::construction_material_cost(infrastructure_type),
        )])
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
    use crate::world::types::{ConstructionLayer, Good, InfrastructureType, Storable};

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
        let mut body = CelestialBodyData {
            orbital_stocks: HashMap::from([(Storable::Good(Good::ConstructionMaterials), 100.0)]),
            ..Default::default()
        };

        infrastructure.queue_build(InfrastructureType::SolarPanel, 2);
        assert_eq!(infrastructure.build_queue.len(), 1);

        infrastructure.process_construction(0.5, 30.0, EntityType::Planet, &mut body);
        assert_eq!(infrastructure.get_count(InfrastructureType::SolarPanel), 0);
        assert_eq!(infrastructure.build_queue.front().unwrap().1, 2);
        assert_eq!(infrastructure.construction_progress, 15.0);

        infrastructure.process_construction(0.5, 30.0, EntityType::Planet, &mut body);
        assert_eq!(infrastructure.get_count(InfrastructureType::SolarPanel), 1);
        assert_eq!(infrastructure.build_queue.front().unwrap().1, 1);

        infrastructure.process_construction(1.0, 30.0, EntityType::Planet, &mut body);
        assert_eq!(infrastructure.get_count(InfrastructureType::SolarPanel), 2);
        assert!(infrastructure.build_queue.is_empty());
        assert_eq!(
            body.stocks_at(ConstructionLayer::Orbit)[&Storable::Good(Good::ConstructionMaterials)],
            40.0
        );
    }

    #[test]
    fn test_get_build_cost() {
        let costs = EntityInfrastructure::get_build_cost(InfrastructureType::Mine);
        assert_eq!(costs.len(), 1);
        assert_eq!(
            costs.get(&Storable::Good(Good::ConstructionMaterials)),
            Some(&50.0)
        );

        let costs = EntityInfrastructure::get_build_cost(InfrastructureType::Farm);
        assert_eq!(costs.len(), 1);
        assert_eq!(
            costs.get(&Storable::Good(Good::ConstructionMaterials)),
            Some(&70.0)
        );

        let costs = EntityInfrastructure::get_build_cost(InfrastructureType::Spaceport);
        assert_eq!(costs.len(), 1);
        assert_eq!(
            costs.get(&Storable::Good(Good::ConstructionMaterials)),
            Some(&100.0)
        );
    }

    #[test]
    fn construction_only_consumes_material_from_the_exact_layer() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure.queue_build(InfrastructureType::SolarPanel, 1);
        let mut body = CelestialBodyData {
            population: 1_000_000.0,
            stocks: HashMap::from([(Storable::Good(Good::ConstructionMaterials), 100.0)]),
            ..Default::default()
        };

        infrastructure.process_construction(30.0, 1.0, EntityType::Planet, &mut body);

        assert_eq!(infrastructure.get_count(InfrastructureType::SolarPanel), 0);
        assert_eq!(
            body.stocks[&Storable::Good(Good::ConstructionMaterials)],
            100.0
        );
        assert_eq!(infrastructure.construction_progress, 0.0);
    }

    #[test]
    fn robots_can_construct_in_a_gas_giants_upper_atmosphere() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure.queue_build(InfrastructureType::Mine, 1);
        let mut body = CelestialBodyData {
            robotic_construction_capacity: 10.0,
            stocks: HashMap::from([(Storable::Good(Good::ConstructionMaterials), 50.0)]),
            ..Default::default()
        };

        let construction_capacity = body.construction_capacity();
        infrastructure.process_construction(
            5.0,
            construction_capacity,
            EntityType::GasGiant,
            &mut body,
        );

        assert_eq!(infrastructure.get_count(InfrastructureType::Mine), 1);
        assert_eq!(
            body.stocks_at(ConstructionLayer::UpperAtmosphere)
                [&Storable::Good(Good::ConstructionMaterials)],
            0.0
        );
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
