#![allow(dead_code)] // TODO remove later

use crate::world::types::{
    CelestialBodyData, ConstructionLayer, EntityType, Good, InfrastructureType, Storable,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// maximum construction material staged by automatic procurement at once.
pub const CONSTRUCTION_STAGING_MATERIAL_LIMIT: f32 = 300.0;

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
    Storage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InfrastructureEffect {
    Mining {
        rate_per_unit: f32,
    },
    FuelCellRefining {
        rate_per_unit: f32,
    },
    FoodProduction {
        rate_per_unit: f32,
    },
    Shipbuilding,
    ConstructionMaterialRefining {
        rate_per_unit: f32,
    },
    ResearchGeneration {
        rate_per_unit: f32,
    },
    Spaceport,
    EnergyGeneration {
        rate_per_unit: f32,
    },
    SurfaceStorage {
        capacity_per_unit: f32,
    },
    UpperAtmosphereStorage {
        capacity_per_unit: f32,
    },
    OrbitalStorage {
        capacity_per_unit: f32,
    },
    OrbitalDock {
        throughput_per_unit: f32,
        berths_per_unit: u32,
    },
}

impl InfrastructureEffect {
    pub fn mining_rate(self) -> Option<f32> {
        match self {
            Self::Mining { rate_per_unit } => Some(rate_per_unit),
            _ => None,
        }
    }

    pub fn fuel_cell_refining_rate(self) -> Option<f32> {
        match self {
            Self::FuelCellRefining { rate_per_unit } => Some(rate_per_unit),
            _ => None,
        }
    }

    pub fn food_production_rate(self) -> Option<f32> {
        match self {
            Self::FoodProduction { rate_per_unit } => Some(rate_per_unit),
            _ => None,
        }
    }

    pub fn construction_material_refining_rate(self) -> Option<f32> {
        match self {
            Self::ConstructionMaterialRefining { rate_per_unit } => Some(rate_per_unit),
            _ => None,
        }
    }

    pub fn research_generation_rate(self) -> Option<f32> {
        match self {
            Self::ResearchGeneration { rate_per_unit } => Some(rate_per_unit),
            _ => None,
        }
    }

    pub fn energy_generation_rate(self) -> Option<f32> {
        match self {
            Self::EnergyGeneration { rate_per_unit } => Some(rate_per_unit),
            _ => None,
        }
    }
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

    pub fn capacity_for(self, count: u32) -> u32 {
        self.capacity_use.saturating_mul(count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InfrastructureCapacity {
    pub limit: u32,
    pub completed: u32,
    pub queued: u32,
}

impl InfrastructureCapacity {
    pub fn allocated(self) -> u32 {
        self.completed.saturating_add(self.queued)
    }

    pub fn remaining(self) -> u32 {
        self.limit.saturating_sub(self.allocated())
    }

    pub fn can_fit(self, required: u32) -> bool {
        required <= self.remaining()
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
const SURFACE_WAREHOUSE_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 100.0,
}];
const UPPER_ATMOSPHERE_STORAGE_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 120.0,
}];
const SPACEPORT_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 100.0,
}];
const SOLAR_PANEL_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 30.0,
}];
const ORBITAL_DEPOT_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 150.0,
}];
const ORBITAL_DOCK_COSTS: &[InfrastructureCost] = &[InfrastructureCost {
    resource: Storable::Good(Good::ConstructionMaterials),
    quantity: 180.0,
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
        infrastructure_type: InfrastructureType::SurfaceWarehouse,
        name: "surface warehouse",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Storage,
        costs: SURFACE_WAREHOUSE_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::SurfaceStorage {
            capacity_per_unit: 1_000.0,
        },
        player_buildable: true,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::UpperAtmosphereStorage,
        name: "upper-atmosphere storage",
        domain: InfrastructureDomain::Ground,
        category: InfrastructureCategory::Storage,
        costs: UPPER_ATMOSPHERE_STORAGE_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::UpperAtmosphereStorage {
            capacity_per_unit: 1_000.0,
        },
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
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::OrbitalDepot,
        name: "orbital depot",
        domain: InfrastructureDomain::Orbit,
        category: InfrastructureCategory::Storage,
        costs: ORBITAL_DEPOT_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::OrbitalStorage {
            capacity_per_unit: 1_000.0,
        },
        player_buildable: true,
    },
    InfrastructureDefinition {
        infrastructure_type: InfrastructureType::OrbitalDock,
        name: "orbital dock",
        domain: InfrastructureDomain::Orbit,
        category: InfrastructureCategory::Logistics,
        costs: ORBITAL_DOCK_COSTS,
        capacity_use: 1,
        effect: InfrastructureEffect::OrbitalDock {
            throughput_per_unit: 100.0,
            berths_per_unit: 1,
        },
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

    /// capacity occupied by completed infrastructure.
    pub fn completed_capacity_use(&self) -> u32 {
        self.infra.iter().fold(0, |capacity, (kind, count)| {
            capacity.saturating_add(kind.definition().capacity_for(*count))
        })
    }

    /// capacity reserved by all queued infrastructure.
    pub fn queued_capacity_use(&self) -> u32 {
        self.build_queue.iter().fold(0, |capacity, (kind, count)| {
            capacity.saturating_add(kind.definition().capacity_for(*count))
        })
    }

    /// summed rate for completed infrastructure matching an effect selector.
    pub fn effect_rate(&self, rate_for: impl Fn(InfrastructureEffect) -> Option<f32>) -> f32 {
        infrastructure_definitions()
            .iter()
            .filter_map(|definition| {
                rate_for(definition.effect)
                    .map(|rate| self.get_count(definition.infrastructure_type) as f32 * rate)
            })
            .sum()
    }

    /// completed units in a catalog category.
    pub fn completed_units_in_category(&self, category: InfrastructureCategory) -> u32 {
        infrastructure_definitions()
            .iter()
            .filter(|definition| definition.category == category)
            .fold(0, |units, definition| {
                units.saturating_add(self.get_count(definition.infrastructure_type))
            })
    }

    /// storage capacity supplied for one logistics layer.
    pub fn storage_capacity(&self, layer: ConstructionLayer) -> f32 {
        infrastructure_definitions()
            .iter()
            .filter_map(|definition| {
                let capacity_per_unit = match (definition.effect, layer) {
                    (
                        InfrastructureEffect::SurfaceStorage { capacity_per_unit },
                        ConstructionLayer::Surface,
                    )
                    | (
                        InfrastructureEffect::UpperAtmosphereStorage { capacity_per_unit },
                        ConstructionLayer::UpperAtmosphere,
                    )
                    | (
                        InfrastructureEffect::OrbitalStorage { capacity_per_unit },
                        ConstructionLayer::Orbit,
                    ) => capacity_per_unit,
                    _ => return None,
                };
                Some(self.get_count(definition.infrastructure_type) as f32 * capacity_per_unit)
            })
            .sum()
    }

    /// completed orbital unloading throughput per economy interval.
    pub fn orbital_dock_throughput(&self) -> f32 {
        infrastructure_definitions()
            .iter()
            .filter_map(|definition| match definition.effect {
                InfrastructureEffect::OrbitalDock {
                    throughput_per_unit,
                    ..
                } => Some(
                    self.get_count(definition.infrastructure_type) as f32 * throughput_per_unit,
                ),
                _ => None,
            })
            .sum()
    }

    /// completed orbital berth capacity.
    pub fn orbital_berth_capacity(&self) -> u32 {
        infrastructure_definitions()
            .iter()
            .fold(0, |berths, definition| match definition.effect {
                InfrastructureEffect::OrbitalDock {
                    berths_per_unit, ..
                } => berths.saturating_add(
                    self.get_count(definition.infrastructure_type)
                        .saturating_mul(berths_per_unit),
                ),
                _ => berths,
            })
    }

    /// near-term construction material target for one layer, preserving queue order.
    pub fn staged_construction_material(
        &self,
        anchor_type: EntityType,
        requested_layer: ConstructionLayer,
    ) -> f32 {
        let mut horizon = CONSTRUCTION_STAGING_MATERIAL_LIMIT;
        let mut staged = 0.0;
        let mut first_unit = true;

        for (infrastructure_type, count) in &self.build_queue {
            let Some(layer) = infrastructure_type.construction_layer(anchor_type) else {
                continue;
            };
            let unit_cost = infrastructure_type
                .definition()
                .construction_material_cost();
            let mut units = *count;
            while units > 0 && horizon > 0.0 {
                let cost = if first_unit {
                    first_unit = false;
                    (unit_cost - self.construction_progress).max(0.0)
                } else {
                    unit_cost
                };
                let within_horizon = cost.min(horizon);
                if layer == requested_layer {
                    staged += within_horizon;
                }
                horizon -= within_horizon;
                units -= 1;
            }
            if horizon == 0.0 {
                break;
            }
        }
        staged
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
            let material_resource = Storable::Good(Good::ConstructionMaterials);
            let material = body_data.amount_at(layer, material_resource);
            let material_needed = (material_cost - self.construction_progress).max(0.0);
            let progress = remaining_capacity.min(material).min(material_needed);

            if progress == 0.0 {
                break;
            }

            body_data.withdraw_at(layer, material_resource, progress);
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
                InfrastructureType::SurfaceWarehouse,
                InfrastructureType::UpperAtmosphereStorage,
                InfrastructureType::Spaceport,
                InfrastructureType::SolarPanel,
                InfrastructureType::OrbitalDepot,
                InfrastructureType::OrbitalDock,
            ]
        );
        assert_eq!(
            player_buildable_infrastructure()
                .map(|definition| definition.infrastructure_type)
                .collect::<Vec<_>>(),
            vec![
                InfrastructureType::SurfaceWarehouse,
                InfrastructureType::Spaceport,
                InfrastructureType::SolarPanel,
                InfrastructureType::OrbitalDepot,
                InfrastructureType::OrbitalDock,
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

    #[test]
    fn capacity_use_counts_completed_and_queued_units_from_catalog() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure.infra.insert(InfrastructureType::Shipyard, 2);
        infrastructure
            .infra
            .insert(InfrastructureType::SolarPanel, 1);
        infrastructure.queue_build(InfrastructureType::Shipyard, 1);
        infrastructure.queue_build(InfrastructureType::SolarPanel, 3);

        assert_eq!(infrastructure.completed_capacity_use(), 5);
        assert_eq!(infrastructure.queued_capacity_use(), 5);
    }

    #[test]
    fn capacity_snapshot_reserves_queued_capacity() {
        let capacity = InfrastructureCapacity {
            limit: 8,
            completed: 3,
            queued: 4,
        };

        assert_eq!(capacity.allocated(), 7);
        assert_eq!(capacity.remaining(), 1);
        assert!(capacity.can_fit(1));
        assert!(!capacity.can_fit(2));
    }

    #[test]
    fn effect_rates_and_categories_follow_catalog_definitions() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure.infra.insert(InfrastructureType::Mine, 3);
        infrastructure
            .infra
            .insert(InfrastructureType::SolarPanel, 2);
        infrastructure.infra.insert(InfrastructureType::Shipyard, 1);

        assert_eq!(
            infrastructure.effect_rate(InfrastructureEffect::mining_rate),
            3.0
        );
        assert_eq!(
            infrastructure.effect_rate(InfrastructureEffect::energy_generation_rate),
            2.0
        );
        assert_eq!(
            infrastructure.completed_units_in_category(InfrastructureCategory::Shipbuilding),
            1
        );
    }

    #[test]
    fn catalog_exposes_all_v1_categories_in_order() {
        assert_eq!(
            infrastructure_definitions()
                .iter()
                .map(|definition| definition.category)
                .collect::<Vec<_>>(),
            vec![
                InfrastructureCategory::Mining,
                InfrastructureCategory::Manufacturing,
                InfrastructureCategory::Agriculture,
                InfrastructureCategory::Shipbuilding,
                InfrastructureCategory::Construction,
                InfrastructureCategory::Research,
                InfrastructureCategory::Storage,
                InfrastructureCategory::Storage,
                InfrastructureCategory::Logistics,
                InfrastructureCategory::Energy,
                InfrastructureCategory::Storage,
                InfrastructureCategory::Logistics,
            ]
        );
    }

    #[test]
    fn storage_and_dock_effects_derive_capacity_and_throughput() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure
            .infra
            .insert(InfrastructureType::SurfaceWarehouse, 2);
        infrastructure
            .infra
            .insert(InfrastructureType::UpperAtmosphereStorage, 3);
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDepot, 4);
        infrastructure
            .infra
            .insert(InfrastructureType::OrbitalDock, 2);

        assert_eq!(
            infrastructure.storage_capacity(ConstructionLayer::Surface),
            2_000.0
        );
        assert_eq!(
            infrastructure.storage_capacity(ConstructionLayer::UpperAtmosphere),
            3_000.0
        );
        assert_eq!(
            infrastructure.storage_capacity(ConstructionLayer::Orbit),
            4_000.0
        );
        assert_eq!(infrastructure.orbital_dock_throughput(), 200.0);
        assert_eq!(infrastructure.orbital_berth_capacity(), 2);
    }

    #[test]
    fn staged_construction_demand_is_bounded_and_layer_exact() {
        let mut infrastructure = EntityInfrastructure::new("test");
        infrastructure.queue_build(InfrastructureType::SolarPanel, 1);
        infrastructure.queue_build(InfrastructureType::Mine, 1);
        infrastructure.queue_build(InfrastructureType::Spaceport, 3);
        infrastructure.construction_progress = 10.0;

        assert_eq!(
            infrastructure
                .staged_construction_material(EntityType::Planet, ConstructionLayer::Surface),
            50.0
        );
        assert_eq!(
            infrastructure
                .staged_construction_material(EntityType::Planet, ConstructionLayer::Orbit),
            250.0
        );
    }
}
