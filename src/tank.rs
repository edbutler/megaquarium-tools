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

#[derive(Debug)]
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

impl std::fmt::Display for TankModel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(tank-model")?;
        write!(f, "\n  #:id {}", self.id)?;
        write!(f, "\n  #:min-size ({} {})", self.min_size.0, self.min_size.1)?;
        write!(f, "\n  #:max-size ({} {})", self.max_size.0, self.max_size.1)?;
        write!(f, "\n  #:density {}", self.density())?;
        if self.rounded {
            write!(f, "\n  #:rounded #t")?;
        }
        write!(f, "))\n")?;
        Ok(())
    }
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

#[derive(Debug, Copy, Clone)]
pub enum Temperature {
    Warm,
    Cold,
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Temperature::Warm => write!(f, "warm"),
            Temperature::Cold => write!(f, "cold"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Salinity {
    Fresh,
    Salty,
}
