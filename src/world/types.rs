use std::collections::HashMap;

/// the types of resources that can be extracted from celestial bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceType {
    // planetary
    Metals,
    Organics,
    Crystals,
    Isotopes,
    Microbes,

    // gas giants
    Volatiles,
    RareExotics,
    DarkMatter,
    NobleGases,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Star,
    Planet,
    Moon,
    GasGiant,
    Ship,
}

pub const PLANETARY_RESOURCES: &[ResourceType] = &[
    ResourceType::Metals,
    ResourceType::Organics,
    ResourceType::Crystals,
    ResourceType::Isotopes,
    ResourceType::Microbes,
];

pub const GAS_GIANT_RESOURCES: &[ResourceType] = &[
    ResourceType::Volatiles,
    ResourceType::RareExotics,
    ResourceType::DarkMatter,
    ResourceType::NobleGases,
];

/// data specific to celestial bodies, such as population and resource yields.
#[derive(Debug, Clone, Default)]
pub struct CelestialBodyData {
    /// the population of the celestial body, which acts as a multiplier for resource extraction.
    pub population: f32,
    /// a map of resource types to their yield grades. the yield grade is a multiplier for resource extraction.
    pub yields: HashMap<ResourceType, f32>,
    /// a map of resource types to their current stock on the celestial body.
    pub stocks: HashMap<ResourceType, f32>,
    /// credits held by the civilian economy on this body.
    pub credits: f64,
}
