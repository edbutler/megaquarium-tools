#[derive(Debug, Copy, Clone)]
pub struct Tank<'a> {
    pub id: u64,
    pub model: &'a TankModel,
    pub size: (u16, u16),
}

impl Tank<'_> {
    pub fn volume(&self) -> u16 {
        self.size.0 * self.size.1 * self.model.double_density / 2
    }
}

#[derive(Debug)]
pub struct TankModel {
    pub id: String,
    pub min_size: (u16, u16),
    pub max_size: (u16, u16),
    // some tanks have, e.g., 3.5 vol/tile, so we store double density to avoid floats
    pub double_density: u16,
    pub rounded: bool,
}

#[derive(Debug)]
pub struct TankSpec {
    pub model: String,
    pub size: (u16, u16),
}

#[derive(Debug)]
pub struct Environment {
    pub temperature: Temperature,
    pub salinity: Salinity,
    pub quality: u8,
}

#[derive(Debug)]
pub enum Temperature {
    Warm,
    Cold,
}

#[derive(Debug)]
pub enum Salinity {
    Fresh,
    Salty,
}
