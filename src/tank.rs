use crate::{util::{as_str_display, Result}, data::GameData};

pub type TankId = u64;

#[derive(Debug, Clone)]
pub struct Tank {
    pub id: TankId,
    pub model: String,
    pub size: (u16, u16),
}

impl Tank {
    pub fn to_ref<'a>(&self, data: &'a GameData) -> Result<TankRef<'a>> {
        let model = data.tank_ref(&self.model)?;
        Ok(TankRef {
            id: self.id,
            model,
            size: self.size
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TankRef<'a> {
    pub id: u64,
    pub model: &'a TankModel,
    pub size: (u16, u16),
}

impl TankRef<'_> {
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

/// Properties of a tank. a None value for a property means "unconstrained," in that any animal with a
/// need for that property will not be satisfied by this tank.
#[derive(Debug, Copy, Clone)]
pub struct Environment {
    pub size: u16,
    pub temperature: Temperature,
    pub quality: u8,
    pub plants: Option<u16>,
    pub rocks: Option<u16>,
    pub caves: Option<u16>,
    pub light: Option<u8>,
    pub interior: Option<Interior>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Interior {
    Rounded,
    Kreisel,
}

impl Interior {
    pub fn as_str(&self) -> &'static str {
        match self {
            Interior::Rounded => "rounded",
            Interior::Kreisel => "kreisel",
        }
    }
}

as_str_display!(Interior);

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

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn test_environment() -> Environment {
        Environment {
            size: 0,
            temperature: Temperature::Warm,
            quality: 0,
            plants: None,
            rocks: None,
            caves: None,
            light: None,
            interior: None,
        }
    }
}
