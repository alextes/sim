use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The types of resources that can be extracted from celestial bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ResourceType {
    Metal,
    Nobles,
    Organics,
}

/// Data specific to celestial bodies, such as population and resource yields.
#[derive(Debug, Clone, Default)]
pub struct CelestialBodyData {
    /// The population of the celestial body, which acts as a multiplier for resource extraction.
    pub population: f32,
    /// A map of resource types to their yield grades. The yield grade is a multiplier for resource extraction.
    pub yields: HashMap<ResourceType, f32>,
}