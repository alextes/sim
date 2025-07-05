#![allow(dead_code)] // TODO remove later

use crate::world::types::{RawResource, Storable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of infrastructure that can be constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InfrastructureType {
    Mine,
    FuelCellCracker,
    Farm,
    Shipyard,
    SolarPanel,
    Construction,
}

/// Returns the cost to build one unit of a given infrastructure type.
pub fn get_infra_build_costs(infra_type: InfrastructureType) -> HashMap<Storable, f32> {
    let mut costs = HashMap::new();
    match infra_type {
        InfrastructureType::Mine => {
            costs.insert(Storable::Raw(RawResource::Metals), 50.0);
        }
        InfrastructureType::FuelCellCracker => {
            costs.insert(Storable::Raw(RawResource::Metals), 100.0);
            costs.insert(Storable::Raw(RawResource::Crystals), 75.0);
        }
        InfrastructureType::Farm => {
            costs.insert(Storable::Raw(RawResource::Metals), 20.0);
            costs.insert(Storable::Raw(RawResource::Organics), 50.0);
        }
        InfrastructureType::Shipyard => {
            costs.insert(Storable::Raw(RawResource::Metals), 200.0);
            costs.insert(Storable::Raw(RawResource::Crystals), 150.0);
        }
        InfrastructureType::SolarPanel => {
            costs.insert(Storable::Raw(RawResource::Metals), 30.0);
            costs.insert(Storable::Raw(RawResource::Crystals), 20.0);
        }
        InfrastructureType::Construction => {
            costs.insert(Storable::Raw(RawResource::Metals), 80.0);
            costs.insert(Storable::Raw(RawResource::Crystals), 40.0);
        }
    }
    costs
}

/// Helper to get a display name for an infrastructure type.
pub fn get_infra_name(infra_type: InfrastructureType) -> &'static str {
    match infra_type {
        InfrastructureType::Mine => "mine",
        InfrastructureType::FuelCellCracker => "fuel cell cracker",
        InfrastructureType::Farm => "farm",
        InfrastructureType::Shipyard => "shipyard",
        InfrastructureType::SolarPanel => "solar panel",
        InfrastructureType::Construction => "construction",
    }
}

// Constants for slot counts
pub const PLANET_SLOTS: usize = 4;
pub const MOON_SLOTS: usize = 2;
pub const GAS_GIANT_SLOTS: usize = 8;
