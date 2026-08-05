use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// the types of resources that can be extracted from celestial bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RawResource {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Good {
    /// synthetic fuel for standard ship drives.
    FuelCells,
    /// universal feedstock consumed by local construction projects.
    ConstructionMaterials,
    Food,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Storable {
    Raw(RawResource),
    Good(Good),
}

impl fmt::Display for RawResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for Good {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for Storable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Storable::Raw(r) => write!(f, "{r}"),
            Storable::Good(g) => write!(f, "{g}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Star,
    Planet,
    Moon,
    GasGiant,
    Ship,
}

/// environment-local layers that can hold stocks and host construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConstructionLayer {
    Surface,
    UpperAtmosphere,
    Orbit,
}

impl ConstructionLayer {
    /// returns the layers where construction can exist for an anchor entity.
    pub fn available_for(entity_type: EntityType) -> &'static [Self] {
        match entity_type {
            EntityType::Planet | EntityType::Moon => &[Self::Surface, Self::Orbit],
            EntityType::GasGiant => &[Self::UpperAtmosphere, Self::Orbit],
            EntityType::Star => &[Self::Orbit],
            EntityType::Ship => &[],
        }
    }

    /// returns the non-orbital layer used by local extraction and industry.
    pub fn primary_for(entity_type: EntityType) -> Option<Self> {
        match entity_type {
            EntityType::Planet | EntityType::Moon => Some(Self::Surface),
            EntityType::GasGiant => Some(Self::UpperAtmosphere),
            EntityType::Star | EntityType::Ship => None,
        }
    }
}

impl fmt::Display for ConstructionLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Surface => write!(f, "surface"),
            Self::UpperAtmosphere => write!(f, "upper atmosphere"),
            Self::Orbit => write!(f, "orbit"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyClass {
    Greenhouse,
    Barren,
    Volcanic,
    Oceanic,
    Lunar,
    GasGiant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodySize {
    Tiny,
    Small,
    Medium,
    Large,
    Giant,
}

impl BodySize {
    #[allow(dead_code)]
    pub fn capacity(self) -> u32 {
        match self {
            BodySize::Tiny => 4,
            BodySize::Small => 8,
            BodySize::Medium => 16,
            BodySize::Large => 28,
            BodySize::Giant => 48,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Atmosphere {
    None,
    Thin,
    Breathable,
    Dense,
    Toxic,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodyProfile {
    pub class: BodyClass,
    pub size: BodySize,
    pub gravity: f32,
    pub atmosphere: Atmosphere,
}

impl BodyProfile {
    #[allow(dead_code)]
    pub fn capacity(self) -> u32 {
        self.size.capacity()
    }

    pub fn default_planet() -> Self {
        Self {
            class: BodyClass::Barren,
            size: BodySize::Medium,
            gravity: 1.0,
            atmosphere: Atmosphere::Thin,
        }
    }

    pub fn default_moon() -> Self {
        Self {
            class: BodyClass::Lunar,
            size: BodySize::Small,
            gravity: 0.2,
            atmosphere: Atmosphere::None,
        }
    }

    pub fn default_gas_giant() -> Self {
        Self {
            class: BodyClass::GasGiant,
            size: BodySize::Giant,
            gravity: 2.5,
            atmosphere: Atmosphere::Dense,
        }
    }
}

/// raw resources used by the v1 progression loop.
pub const V1_RAW_RESOURCES: &[RawResource] = &[
    RawResource::Metals,
    RawResource::Organics,
    RawResource::Crystals,
    RawResource::Volatiles,
];

pub const PLANETARY_RESOURCES: &[RawResource] = &[
    RawResource::Metals,
    RawResource::Organics,
    RawResource::Crystals,
];

pub const GAS_GIANT_RESOURCES: &[RawResource] = &[RawResource::Volatiles];

/// data specific to celestial bodies, such as population and resource yields.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CelestialBodyData {
    /// credits held by the civilian economy on this body.
    pub credits: f64,
    /// the population of the celestial body, which acts as a multiplier for resource extraction.
    pub population: f32,
    /// a map of resource types to their yield grades. the yield grade is a multiplier for resource extraction.
    pub yields: HashMap<RawResource, f32>,
    /// stocks in the body's primary layer: surface for solid bodies and upper atmosphere for gas giants.
    pub stocks: HashMap<Storable, f32>,
    /// stocks delivered to this body's orbital layer.
    #[serde(default)]
    pub orbital_stocks: HashMap<Storable, f32>,
    /// construction throughput supplied by robots, in material units per second.
    #[serde(default)]
    pub robotic_construction_capacity: f32,
    /// a map of raw resource types to their monthly demand on the celestial body.
    pub demands: HashMap<Storable, f32>,
}

impl CelestialBodyData {
    /// biological and robotic construction throughput in material units per second.
    pub fn construction_capacity(&self) -> f32 {
        const POPULATION_PER_CAPACITY: f32 = 1_000_000.0;
        (self.population / POPULATION_PER_CAPACITY).max(0.0)
            + self.robotic_construction_capacity.max(0.0)
    }

    /// returns stocks at an environment-local layer.
    pub fn stocks_at(&self, layer: ConstructionLayer) -> &HashMap<Storable, f32> {
        match layer {
            ConstructionLayer::Surface | ConstructionLayer::UpperAtmosphere => &self.stocks,
            ConstructionLayer::Orbit => &self.orbital_stocks,
        }
    }

    /// returns mutable stocks at an environment-local layer.
    fn stocks_at_mut(&mut self, layer: ConstructionLayer) -> &mut HashMap<Storable, f32> {
        match layer {
            ConstructionLayer::Surface | ConstructionLayer::UpperAtmosphere => &mut self.stocks,
            ConstructionLayer::Orbit => &mut self.orbital_stocks,
        }
    }

    /// resource units currently stored at one logistics layer.
    pub fn stored_units_at(&self, layer: ConstructionLayer) -> f32 {
        let mut stocks: Vec<_> = self.stocks_at(layer).iter().collect();
        stocks.sort_by_key(|(storable, _)| **storable);
        stocks.into_iter().map(|(_, amount)| *amount).sum()
    }

    /// amount of one resource stored at a logistics layer.
    pub fn amount_at(&self, layer: ConstructionLayer, resource: Storable) -> f32 {
        self.stocks_at(layer).get(&resource).copied().unwrap_or(0.0)
    }

    /// free shared storage capacity at a logistics layer.
    #[allow(dead_code)] // used by the next bounded-deposit slice
    pub fn free_capacity_at(&self, layer: ConstructionLayer, capacity: f32) -> f32 {
        (capacity.max(0.0) - self.stored_units_at(layer)).max(0.0)
    }

    /// deposits as much as fits and returns the accepted quantity.
    #[allow(dead_code)] // used by the next bounded-deposit slice
    pub fn deposit_bounded_at(
        &mut self,
        layer: ConstructionLayer,
        resource: Storable,
        quantity: f32,
        capacity: f32,
    ) -> f32 {
        let accepted = quantity
            .max(0.0)
            .min(self.free_capacity_at(layer, capacity));
        if accepted > 0.0 {
            *self.stocks_at_mut(layer).entry(resource).or_insert(0.0) += accepted;
        }
        accepted
    }

    /// deposits without a capacity limit for migrations and seeded state.
    pub fn deposit_unbounded_at(
        &mut self,
        layer: ConstructionLayer,
        resource: Storable,
        quantity: f32,
    ) -> f32 {
        let accepted = quantity.max(0.0);
        if accepted > 0.0 {
            *self.stocks_at_mut(layer).entry(resource).or_insert(0.0) += accepted;
        }
        accepted
    }

    /// withdraws up to the requested quantity and returns the withdrawn amount.
    pub fn withdraw_at(
        &mut self,
        layer: ConstructionLayer,
        resource: Storable,
        quantity: f32,
    ) -> f32 {
        let requested = quantity.max(0.0);
        let available = self.amount_at(layer, resource);
        let withdrawn = requested.min(available);
        if withdrawn > 0.0 {
            *self.stocks_at_mut(layer).entry(resource).or_insert(0.0) -= withdrawn;
        }
        withdrawn
    }

    /// stocks at a layer in deterministic resource order.
    pub fn ordered_stocks_at(&self, layer: ConstructionLayer) -> Vec<(Storable, f32)> {
        let mut stocks: Vec<_> = self
            .stocks_at(layer)
            .iter()
            .map(|(resource, amount)| (*resource, *amount))
            .collect();
        stocks.sort_by_key(|(resource, _)| *resource);
        stocks
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const STAR_COLORS: [Color; 3] = [
    Color {
        r: 255,
        g: 255,
        b: 255,
    }, // white
    Color {
        r: 255,
        g: 255,
        b: 224,
    }, // yellow-white
    Color {
        r: 173,
        g: 216,
        b: 230,
    }, // pale blue
];

pub const PLANET_COLORS: [Color; 3] = [
    Color {
        r: 60,
        g: 179,
        b: 113,
    }, // blue-green
    Color {
        r: 183,
        g: 65,
        b: 14,
    }, // rusty red
    Color {
        r: 244,
        g: 164,
        b: 96,
    }, // sandy brown
];

pub const MOON_COLORS: [Color; 3] = [
    Color {
        r: 211,
        g: 211,
        b: 211,
    }, // light gray
    Color {
        r: 128,
        g: 128,
        b: 128,
    }, // gray
    Color {
        r: 169,
        g: 169,
        b: 169,
    }, // dark gray
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InfrastructureType {
    // ground
    Mine,
    FuelCellCracker,
    Farm,
    Shipyard,
    ConstructionFactory,
    ResearchLab,
    SurfaceWarehouse,
    UpperAtmosphereStorage,
    // orbital
    Spaceport,
    SolarPanel,
    OrbitalDepot,
    OrbitalDock,
}

pub const MAX_SPACEPORT_UNITS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceportSize {
    Small,
    Medium,
    Large,
}

impl SpaceportSize {
    pub fn from_completed_units(units: u32) -> Option<Self> {
        match units {
            0 => None,
            1 => Some(Self::Small),
            2 => Some(Self::Medium),
            _ => Some(Self::Large),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spaceport {
    pub name: String,
    pub size: SpaceportSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_resources_and_environment_layers_are_explicit() {
        assert_eq!(
            V1_RAW_RESOURCES,
            &[
                RawResource::Metals,
                RawResource::Organics,
                RawResource::Crystals,
                RawResource::Volatiles,
            ]
        );
        assert_eq!(
            ConstructionLayer::available_for(EntityType::Planet),
            &[ConstructionLayer::Surface, ConstructionLayer::Orbit]
        );
        assert_eq!(
            ConstructionLayer::available_for(EntityType::GasGiant),
            &[ConstructionLayer::UpperAtmosphere, ConstructionLayer::Orbit]
        );
        assert_eq!(
            ConstructionLayer::available_for(EntityType::Star),
            &[ConstructionLayer::Orbit]
        );
    }

    #[test]
    fn construction_capacity_combines_population_and_robots() {
        let body = CelestialBodyData {
            population: 2_000_000.0,
            robotic_construction_capacity: 3.5,
            ..Default::default()
        };

        assert_eq!(body.construction_capacity(), 5.5);
    }

    #[test]
    fn layer_inventory_bounds_deposits_and_withdrawals() {
        let resource = Storable::Raw(RawResource::Metals);
        let mut body = CelestialBodyData::default();

        assert_eq!(
            body.deposit_bounded_at(ConstructionLayer::Surface, resource, 8.0, 5.0),
            5.0
        );
        assert_eq!(body.free_capacity_at(ConstructionLayer::Surface, 5.0), 0.0);
        assert_eq!(
            body.deposit_bounded_at(ConstructionLayer::Surface, resource, 1.0, 5.0),
            0.0
        );
        assert_eq!(
            body.withdraw_at(ConstructionLayer::Surface, resource, 3.0),
            3.0
        );
        assert_eq!(body.amount_at(ConstructionLayer::Surface, resource), 2.0);
        assert_eq!(body.amount_at(ConstructionLayer::Orbit, resource), 0.0);

        let mut gas_body = CelestialBodyData::default();
        gas_body.deposit_unbounded_at(ConstructionLayer::UpperAtmosphere, resource, 4.0);
        body.deposit_unbounded_at(ConstructionLayer::Orbit, resource, 7.0);
        assert_eq!(
            gas_body.amount_at(ConstructionLayer::UpperAtmosphere, resource),
            4.0
        );
        assert_eq!(body.amount_at(ConstructionLayer::Orbit, resource), 7.0);
    }
}
