#![allow(dead_code)] // TODO remove later

use crate::infrastructure::{EntityInfrastructure, InfrastructureEffect};
use crate::world::types::EntityType;
use crate::world::types::{CelestialBodyData, Good, RawResource, Storable};
use crate::world::EntityId;
use crate::world::World;
use crate::SIMULATION_DT;
use std::collections::HashMap;
use std::sync::LazyLock;

// calculate simulation frequency based on simulation DT
static SIMULATION_HZ: LazyLock<f64> = LazyLock::new(|| 1.0 / SIMULATION_DT.as_secs_f64());

// --- resource generation config ---
// generate resources every n seconds of simulated time.
// simulation_dt is 10ms (0.01s), so 100 ticks = 1.0 second.
pub const RESOURCE_INTERVAL_SECONDS: f64 = 1.0; // update once per second

fn process_recipe(
    body: &mut CelestialBodyData,
    layer: crate::world::types::ConstructionLayer,
    storage_capacity: f32,
    inputs: &[(Storable, f32)],
    output: Storable,
    requested_batches: f32,
) -> f32 {
    let actual_batches = inputs
        .iter()
        .fold(requested_batches, |possible, (input, amount)| {
            possible.min(body.amount_at(layer, *input) / amount)
        });

    if actual_batches <= 0.0 {
        return 0.0;
    }

    for (input, amount) in inputs {
        body.withdraw_at(layer, *input, actual_batches * amount);
    }
    body.deposit_bounded_at(layer, output, actual_batches, storage_capacity)
}

#[derive(Debug, Default)]
pub struct ResourceSystem {
    // this system is now only responsible for ticking time forward for production.
    // all resource stockpiles are held in celestial body data.
    time_accumulator: f64, // accumulates dt_seconds
}

impl ResourceSystem {
    /// updates resource counts based on infrastructure and elapsed simulated time.
    pub fn update(
        &mut self,
        dt_seconds: f64, // delta time for the current simulation step
        entity_types: &HashMap<EntityId, EntityType>,
        infrastructure_map: &HashMap<EntityId, EntityInfrastructure>,
        celestial_data_map: &mut HashMap<EntityId, CelestialBodyData>,
    ) {
        self.time_accumulator += dt_seconds;

        let num_intervals = (self.time_accumulator / RESOURCE_INTERVAL_SECONDS).floor() as u32;

        if num_intervals == 0 {
            return;
        }

        self.time_accumulator -= num_intervals as f64 * RESOURCE_INTERVAL_SECONDS;

        let production_multiplier = num_intervals as f32 * RESOURCE_INTERVAL_SECONDS as f32;

        for (entity_id, celestial_data) in celestial_data_map.iter_mut() {
            let entity_type = match entity_types.get(entity_id) {
                Some(t) => t,
                None => continue,
            };

            match entity_type {
                EntityType::Planet | EntityType::Moon | EntityType::GasGiant => {
                    // this entity type produces resources.
                }
                _ => continue, // other types do not produce resources.
            }
            let Some(layer) = crate::world::types::ConstructionLayer::primary_for(*entity_type)
            else {
                continue;
            };

            let infrastructure = match infrastructure_map.get(entity_id) {
                Some(infrastructure) => infrastructure,
                None => continue,
            };
            let storage_capacity = infrastructure.storage_capacity(layer);

            // handle raw resource extraction from mining effects
            let mining_rate = infrastructure.effect_rate(InfrastructureEffect::mining_rate);

            if mining_rate > 0.0 {
                let mut yields: Vec<_> = celestial_data
                    .yields
                    .iter()
                    .map(|(resource, grade)| (*resource, *grade))
                    .collect();
                yields.sort_by_key(|(resource, _)| *resource);
                for (resource_type, yield_grade) in yields {
                    let production = (celestial_data.population / 1_000_000.0)
                        * mining_rate
                        * yield_grade
                        * production_multiplier;
                    celestial_data.deposit_bounded_at(
                        layer,
                        Storable::Raw(resource_type),
                        production,
                        storage_capacity,
                    );
                }
            }

            // handle manufactured goods production
            let fuel_cell_refining_rate =
                infrastructure.effect_rate(InfrastructureEffect::fuel_cell_refining_rate);

            if fuel_cell_refining_rate > 0.0 {
                // recipe: 1 volatile + 0.1 metals -> 1 fuel cell
                process_recipe(
                    celestial_data,
                    layer,
                    storage_capacity,
                    &[
                        (Storable::Raw(RawResource::Volatiles), 1.0),
                        (Storable::Raw(RawResource::Metals), 0.1),
                    ],
                    Storable::Good(Good::FuelCells),
                    fuel_cell_refining_rate * production_multiplier,
                );
            }

            // construction factories refine the universal construction feedstock.
            let construction_material_refining_rate = infrastructure
                .effect_rate(InfrastructureEffect::construction_material_refining_rate);

            if construction_material_refining_rate > 0.0 {
                // recipe: 1 metals + 1 crystals -> 1 construction material
                process_recipe(
                    celestial_data,
                    layer,
                    storage_capacity,
                    &[
                        (Storable::Raw(RawResource::Metals), 1.0),
                        (Storable::Raw(RawResource::Crystals), 1.0),
                    ],
                    Storable::Good(Good::ConstructionMaterials),
                    construction_material_refining_rate * production_multiplier,
                );
            }

            // handle food production from farms
            let food_production_rate =
                infrastructure.effect_rate(InfrastructureEffect::food_production_rate);

            if food_production_rate > 0.0 {
                // recipe: 1 organics -> 1 food
                process_recipe(
                    celestial_data,
                    layer,
                    storage_capacity,
                    &[(Storable::Raw(RawResource::Organics), 1.0)],
                    Storable::Good(Good::Food),
                    food_production_rate * production_multiplier,
                );
            }
        }
    }

    /// calculate the current production rates for all resources across all celestial bodies.
    /// this is an aggregate view for the UI, not a global stockpile.
    pub fn calculate_rates(
        &self,
        infrastructure_map: &HashMap<EntityId, EntityInfrastructure>,
        celestial_data_map: &HashMap<EntityId, CelestialBodyData>,
    ) -> HashMap<Storable, f32> {
        let mut rates = HashMap::new();

        // calculate rates based on infrastructure.
        for (entity_id, infrastructure) in infrastructure_map.iter() {
            let celestial_data = match celestial_data_map.get(entity_id) {
                Some(data) => data,
                None => continue,
            };

            let mining_rate = infrastructure.effect_rate(InfrastructureEffect::mining_rate);

            if mining_rate > 0.0 {
                for (resource_type, yield_grade) in &celestial_data.yields {
                    let production_rate =
                        (celestial_data.population / 1_000_000.0) * mining_rate * yield_grade;
                    *rates.entry(Storable::Raw(*resource_type)).or_insert(0.0) += production_rate;
                }
            }

            let fuel_cell_refining_rate =
                infrastructure.effect_rate(InfrastructureEffect::fuel_cell_refining_rate);

            if fuel_cell_refining_rate > 0.0 {
                // this is a simplified view. it does not account for input resource availability.
                *rates.entry(Storable::Good(Good::FuelCells)).or_insert(0.0) +=
                    fuel_cell_refining_rate;
            }

            let construction_material_refining_rate = infrastructure
                .effect_rate(InfrastructureEffect::construction_material_refining_rate);

            if construction_material_refining_rate > 0.0 {
                *rates
                    .entry(Storable::Good(Good::ConstructionMaterials))
                    .or_insert(0.0) += construction_material_refining_rate;
            }

            let food_production_rate =
                infrastructure.effect_rate(InfrastructureEffect::food_production_rate);

            if food_production_rate > 0.0 {
                // simplified view, does not account for input availability
                *rates.entry(Storable::Good(Good::Food)).or_insert(0.0) += food_production_rate;
            }
        }
        rates
    }
}

/// returns the dynamic, local credit value for a single unit of a resource on a specific entity.
pub fn get_local_price(world: &World, entity_id: EntityId, resource: Storable) -> f64 {
    let base_price = get_resource_base_price(resource);

    let celestial_data = match world.celestial_data.get(&entity_id) {
        Some(data) => data,
        None => return base_price, // not a celestial body, return base price
    };

    let (stockpile, monthly_demand) = match resource {
        Storable::Raw(raw_resource) => (
            celestial_data.amount_at(crate::world::types::ConstructionLayer::Surface, resource),
            celestial_data
                .demands
                .get(&Storable::Raw(raw_resource))
                .copied()
                .unwrap_or(0.0),
        ),
        Storable::Good(_) => {
            // for now, goods don't have demand, so they trade at base price
            return base_price;
        }
    };

    const BUFFER_MONTHS: f32 = 3.0;
    // add a small epsilon to demand to avoid division by zero if demand is zero
    let demand_for_ratio = monthly_demand + 1e-6;
    let ratio = stockpile / (demand_for_ratio * BUFFER_MONTHS);

    // price is inversely proportional to supply/demand ratio
    let price_modifier = 1.0 / ratio.max(0.1); // prevent extreme multipliers

    (base_price * price_modifier as f64).clamp(base_price * 0.25, base_price * 4.0)
}

/// returns the base credit value for a single unit of a resource.
pub fn get_resource_base_price(resource: Storable) -> f64 {
    match resource {
        Storable::Raw(raw) => match raw {
            RawResource::Metals => 1.0,
            RawResource::Crystals => 5.0,
            RawResource::Organics => 2.0,
            RawResource::Volatiles => 1.5,
            RawResource::Isotopes => 10.0,
            RawResource::RareExotics => 20.0,
            RawResource::Microbes => 3.0,
            RawResource::DarkMatter => 100.0,
            RawResource::NobleGases => 4.0,
        },
        Storable::Good(good) => match good {
            Good::FuelCells => 2.0,
            Good::ConstructionMaterials => 6.0,
            Good::Food => 2.5,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::EntityInfrastructure;
    use crate::world::types::{
        CelestialBodyData, EntityType, InfrastructureType, RawResource, Storable,
    };
    use std::collections::HashMap;

    fn create_test_data(
        mines: u32,
    ) -> (
        HashMap<EntityId, EntityType>,
        HashMap<EntityId, EntityInfrastructure>,
        HashMap<EntityId, CelestialBodyData>,
    ) {
        let mut infrastructure_map = HashMap::new();
        let mut infrastructure_data = EntityInfrastructure::new("test");
        let entity_id = 1;

        infrastructure_data
            .infra
            .insert(InfrastructureType::Mine, mines);
        infrastructure_data
            .infra
            .insert(InfrastructureType::SurfaceWarehouse, 1);
        infrastructure_map.insert(entity_id, infrastructure_data);

        let mut celestial_data_map = HashMap::new();
        let mut yields = HashMap::new();
        yields.insert(RawResource::Metals, 1.2);
        yields.insert(RawResource::Crystals, 0.4);
        yields.insert(RawResource::Organics, 0.8);

        celestial_data_map.insert(
            entity_id,
            CelestialBodyData {
                population: 1_000_000.0,
                yields,
                ..Default::default()
            },
        );

        let mut entity_types = HashMap::new();
        entity_types.insert(entity_id, EntityType::Planet);

        (entity_types, infrastructure_map, celestial_data_map)
    }

    #[test]
    fn test_resource_system_update() {
        let (entity_types, infrastructure, mut celestial_data) = create_test_data(1);
        let mut resource_system = ResourceSystem::default();

        resource_system.update(
            RESOURCE_INTERVAL_SECONDS,
            &entity_types,
            &infrastructure,
            &mut celestial_data,
        );

        let interval_f32 = RESOURCE_INTERVAL_SECONDS as f32;
        let stocks = &celestial_data.get(&1).unwrap().stocks;
        assert_eq!(
            *stocks.get(&Storable::Raw(RawResource::Metals)).unwrap(),
            1.0 * 1.0 * 1.2 * interval_f32
        );
        assert_eq!(
            *stocks.get(&Storable::Raw(RawResource::Crystals)).unwrap(),
            1.0 * 1.0 * 0.4 * interval_f32
        );
        assert_eq!(
            *stocks.get(&Storable::Raw(RawResource::Organics)).unwrap(),
            1.0 * 1.0 * 0.8 * interval_f32
        );
    }

    #[test]
    fn v1_refining_recipes_consume_inputs_and_produce_goods() {
        let (entity_types, mut infrastructure, mut celestial_data) = create_test_data(0);
        let body_infrastructure = infrastructure.get_mut(&1).unwrap();
        body_infrastructure
            .infra
            .insert(InfrastructureType::FuelCellCracker, 1);
        body_infrastructure
            .infra
            .insert(InfrastructureType::ConstructionFactory, 1);
        body_infrastructure
            .infra
            .insert(InfrastructureType::Farm, 1);
        celestial_data.get_mut(&1).unwrap().stocks = HashMap::from([
            (Storable::Raw(RawResource::Volatiles), 1.0),
            (Storable::Raw(RawResource::Metals), 1.1),
            (Storable::Raw(RawResource::Crystals), 1.0),
            (Storable::Raw(RawResource::Organics), 1.0),
        ]);
        let mut resource_system = ResourceSystem::default();

        resource_system.update(
            RESOURCE_INTERVAL_SECONDS,
            &entity_types,
            &infrastructure,
            &mut celestial_data,
        );

        let stocks = &celestial_data[&1].stocks;
        assert_eq!(stocks[&Storable::Raw(RawResource::Volatiles)], 0.0);
        assert_eq!(stocks[&Storable::Raw(RawResource::Metals)], 0.0);
        assert_eq!(stocks[&Storable::Raw(RawResource::Crystals)], 0.0);
        assert_eq!(stocks[&Storable::Raw(RawResource::Organics)], 0.0);
        assert_eq!(stocks[&Storable::Good(Good::FuelCells)], 1.0);
        assert_eq!(stocks[&Storable::Good(Good::ConstructionMaterials)], 1.0);
        assert_eq!(stocks[&Storable::Good(Good::Food)], 1.0);
    }

    #[test]
    fn extraction_truncates_deterministically_at_storage_capacity() {
        let (entity_types, infrastructure, mut celestial_data) = create_test_data(1);
        let body = celestial_data.get_mut(&1).unwrap();
        body.deposit_unbounded_at(
            crate::world::types::ConstructionLayer::Surface,
            Storable::Good(Good::Food),
            999.0,
        );
        let mut resource_system = ResourceSystem::default();

        resource_system.update(
            RESOURCE_INTERVAL_SECONDS,
            &entity_types,
            &infrastructure,
            &mut celestial_data,
        );

        let body = &celestial_data[&1];
        assert_eq!(
            body.stored_units_at(crate::world::types::ConstructionLayer::Surface),
            1_000.0
        );
        assert_eq!(
            body.amount_at(
                crate::world::types::ConstructionLayer::Surface,
                Storable::Raw(RawResource::Metals)
            ),
            1.0
        );
        assert_eq!(
            body.amount_at(
                crate::world::types::ConstructionLayer::Surface,
                Storable::Raw(RawResource::Organics)
            ),
            0.0
        );
    }
}
