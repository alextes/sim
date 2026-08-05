#![allow(dead_code)] // TODO remove later

use crate::world::types::{
    CelestialBodyData, ConstructionLayer, EntityType, Good, InfrastructureType, Storable,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfrastructureDomain {
    Ground,
    Orbit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfrastructureCategory {
    Energy,
    Mining,
    Research,
    Shipbuilding,
    Manufacturing,
    Agriculture,
    Construction,
    Logistics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InfrastructureEffect {
    Mining { rate_per_unit: f32 },
    FuelCellRefining { rate_per_unit: f32 },
    FoodProduction { rate_per_unit: f32 },
    Shipbuilding,
    ConstructionMaterialRefining { rate_per_unit: f32 },
    ResearchGeneration { rate_per_unit: f32 },
    Spaceport,
    EnergyGeneration { rate_per_unit: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfrastructureCost {
    pub resource: Storable,
    pub quantity: f32,
}

impl InfrastructureCost {
    pub fn scaled(self, count: u32) -> Self {
        Self {
            resource: self.resource,
            quantity: self.quantity * count as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfrastructureDefinition {
    pub infrastructure_type: InfrastructureType,
    pub name: &'static str,
    pub domain: InfrastructureDomain,
    pub category: InfrastructureCategory,
    pub costs: &'static [InfrastructureCost],
    pub capacity_use: u32,
    pub effect: InfrastructureEffect,
    pub player_buildable: bool,
}

impl InfrastructureDefinition {
    pub fn scaled_costs(self, count: u32) -> Vec<InfrastructureCost> {
        self.costs.iter().map(|cost| cost.scaled(count)).collect()
    }

    pub fn construction_material_cost(self) -> f32 {
        self.costs
            .iter()
            .find(|cost| cost.resource == Storable::Good(Good::ConstructionMaterials))
            .map(|cost| cost.quantity)
            .unwrap_or(0.0)
    }
}

const MINE_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 50.0,
}];
const FUEL_CELL_CRACKER_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 175.0,
}];
const FARM_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 70.0,
}];
const SHIPYARD_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 200.0,
}];
const CONSTRUCTION_FACTORY_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 150.0,
}];
const RESEARCH_LAB_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 80.0,
}];
const SPACEPORT_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 100.0,
}];
const SOLAR_PANEL_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 30.0,
}];

const INFRASTRUCTURE_DEFINITIONS: &[InfrastructureDefinition] = &[
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::Mine,
        name: "mine",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Mining,
        costs: MINE_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::Mining { rate_per_unit: 1.0 },
        player_buildable: false,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::FuelCellCracker,
        name: "fuel cell cracker",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Manufacturing,
        costs: FUEL_CELL_CRACKER_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::FuelCellRefining { rate_per_unit: 1.0 },
        player_buildable: false,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::Farm,
        name: "farm",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Agriculture,
        costs: FARM_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::FoodProduction { rate_per_unit: 1.0 },
        player_buildable: false,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::Shipyard,
        name: "shipyard",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Shipbuilding,
        costs: SHIPYARD_COSTS,
        capacity_use: 2,
        effect: InfrastructureEffect::Shipbuilding,
        player_buildable: false,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::ConstructionFactory,
        name: "construction factory",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Construction,
        costs: CONSTRUCTION_FACTORY_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::ConstructionMaterialRefining { rate_per_unit: 1.0 },
        player_buildable: false,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::ResearchLab,
        name: "research lab",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Research,
        costs: RESEARCH_LAB_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::ResearchGeneration { rate_per_unit: 1.0 },
        player_buildable: false,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::Spaceport,
        name: "spaceport",
        domain: InfrastructureDomain::Orbit,
        category: InfrastructureCategory::Logistics,
        costs: SPACEPORT_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::Spaceport,
        player_buildable: true,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::SolarPanel,
        name: "orbital solar panel",
        domain: InfrastructureDomain::Orbit,
        category: InfrastructureCategory::Energy,
        costs: SOLAR_PANEL_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::EnergyGeneration { rate_per_unit: 1.0 },
        player_buildable: true,
    },
];

pub fn infrastructure_definitions() -> &'static [InfrastructureDefinition] {
    INFRASTRUCTURE_DEFINITIONS
}

pub fn player_buildable_infrastructure() -> impl Iterator<Item = &'static InfrastructureDefinition>
{
    INFRASTRUCTURE_DEFINITIONS
        .iter()
        .filter(|definition| definition.player_buildable)
}

impl InfrastructureType {
    pub fn definition(self) -> &'static InfrastructureDefinition {
        INFRASTRUCTURE_DEFINITIONS
            .iter()
            .find(|definition| definition.infrastructure_type == self)
            .expect("every infrastructure type must have a definition")
    }

    /// returns the layer where this infrastructure is constructed.
    pub fn construction_layer(self, anchor_type: EntityType) -> Option<ConstructionLayer> {
        let layer = match self.definition().domain {
            InfrastructureDomain::Ground => ConstructionLayer::primary_for(anchor_type)?,
            InfrastructureDomain::Orbit => ConstructionLayer::Orbit,
        };
        ConstructionLayer::available_for(anchor_type)
            .contains(&layer)
            .then_some(layer)
    }
}

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
            let material_cost = infrastructure_type
                .definition()
                .construction_material_cost();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::types::{ConstructionLayer, Good, InfrastructureType, Storable};

    #[test]
    fn catalog_is_complete_and_deterministically_ordered() {
        let infrastructure_types: Vec<_> = infrastructure_definitions()
            .iter()
            .map(|definition| definition.infrastructure_type)
            .collect();

        assert_eq!(
            infrastructure_types,
            vec![
                InfrastructureType::Mine,
                InfrastructureType::FuelCellCracker,
                InfrastructureType::Farm,
                InfrastructureType::Shipyard,
                InfrastructureType::ConstructionFactory,
                InfrastructureType::ResearchLab,
                InfrastructureType::Spaceport,
                InfrastructureType::SolarPanel,
            ]
        );
        assert_eq!(
            player_buildable_infrastructure()
                .map(|definition| definition.infrastructure_type)
                .collect::<Vec<_>>(),
            vec![
                InfrastructureType::Spaceport,
                InfrastructureType::SolarPanel,
            ]
        );
    }

    #[test]
    fn representative_definitions_include_domain_category_capacity_and_effect() {
        let research_lab = InfrastructureType::ResearchLab.definition();
        assert_eq!(research_lab.name, "research lab");
        assert_eq!(research_lab.domain, InfrastructureDomain::Ground);
        assert_eq!(research_lab.category, InfrastructureCategory::Research);
        assert_eq!(research_lab.capacity_use, 1);
        assert_eq!(
            research_lab.effect,
            InfrastructureEffect::ResearchGeneration { rate_per_unit: 1.0 }
        );
        assert_eq!(
            InfrastructureType::ResearchLab.construction_layer(EntityType::Planet),
            Some(ConstructionLayer::Surface)
        );

        let spaceport = InfrastructureType::Spaceport.definition();
        assert_eq!(spaceport.domain, InfrastructureDomain::Orbit);
        assert_eq!(spaceport.category, InfrastructureCategory::Logistics);
        assert_eq!(spaceport.effect, InfrastructureEffect::Spaceport);

        let shipyard = InfrastructureType::Shipyard.definition();
        assert_eq!(shipyard.category, InfrastructureCategory::Shipbuilding);
        assert_eq!(shipyard.capacity_use, 2);
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
    fn catalog_costs_are_ordered_and_scalable() {
        let costs = InfrastructureType::Mine.definition().costs;
        assert_eq!(
            costs,
            &[InfrastructureCost {
                resource: Storable::Good(Good::ConstructionMaterials),
                quantity: 50.0,
            }]
        );

        assert_eq!(
            InfrastructureType::Spaceport.definition().scaled_costs(3),
            vec![InfrastructureCost {
                resource: Storable::Good(Good::ConstructionMaterials),
                quantity: 300.0,
            }]
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
