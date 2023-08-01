use crate::util::as_str_display;

#[derive(Debug, Copy, Clone)]
pub struct Tank<'a> {
    pub id: u64,
    pub model: &'a TankModel,
    pub size: (u16, u16),
}

impl Tank<'_> {
    pub fn volume(&self) -> u16 {
        // TODO should be ceiling, not floor
        self.size.0 * self.size.1 * self.model.double_density / 2
    }
}

#[derive(Debug, Clone)]
pub struct TankModel {
    pub id: String,
    pub min_size: (u16, u16),
    pub max_size: (u16, u16),
    // some tanks have, e.g., 3.5 vol/tile, so we store double density to avoid floats
    pub double_density: u16,
    pub rounded: bool,
}

impl TankModel {
    pub fn density(&self) -> f64 {
        self.double_density as f64 * 0.5
    }
}

#[derive(Debug, Clone)]
pub struct TankStatus {
    pub size: u16,
    pub environment: Environment,
    // lighting it stored separately from environment due to animal constraints being more complex than a simple comparison
    // None means uncontrained, Some means it has to be that value
    pub lighting: Option<u8>,
    pub rounded: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Environment {
    pub temperature: Temperature,
    pub salinity: Salinity,
    pub quality: u8,
    pub plants: u16,
    pub rocks: u16,
    pub caves: u16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Temperature {
    Warm,
    Cold,
}

impl Temperature {
    pub fn other(&self) -> Temperature {
        match self {
            Temperature::Cold => Temperature::Warm,
            Temperature::Warm => Temperature::Cold,
        }
    }
}

impl Temperature {
    pub fn as_str(&self) -> &'static str {
        match self {
            Temperature::Warm => "warm",
            Temperature::Cold => "cold",
        }
    }
}

as_str_display!(Temperature);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Salinity {
    Fresh,
    Salty,
}
