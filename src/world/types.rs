use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Storable {
    Raw(RawResource),
    Good(Good),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Star,
    Planet,
    Moon,
    GasGiant,
    Ship,
}

pub const PLANETARY_RESOURCES: &[RawResource] = &[
    RawResource::Metals,
    RawResource::Organics,
    RawResource::Crystals,
    RawResource::Isotopes,
    RawResource::Microbes,
];

pub const GAS_GIANT_RESOURCES: &[RawResource] = &[
    RawResource::Volatiles,
    RawResource::RareExotics,
    RawResource::DarkMatter,
    RawResource::NobleGases,
];

/// data specific to celestial bodies, such as population and resource yields.
#[derive(Debug, Clone, Default)]
pub struct CelestialBodyData {
    /// the population of the celestial body, which acts as a multiplier for resource extraction.
    pub population: f32,
    /// a map of resource types to their yield grades. the yield grade is a multiplier for resource extraction.
    pub yields: HashMap<RawResource, f32>,
    /// a map of storable types to their current stock on the celestial body.
    pub stocks: HashMap<Storable, f32>,
    /// credits held by the civilian economy on this body.
    pub credits: f64,
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
