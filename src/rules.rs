use crate::tank;
use Constraint::*;

pub enum Constraint {
    Temperature(tank::Temperature),
    Quality(u8),
    NeedsFood { kind: String, daily_amount: u16 },
    Scavenger,
    Shoaler(u8),
    IsBully,
    NoBully,
    NoLight,
    NeedsLight(u8),
    OnlyGenus(String),
    NoGenus(String),
    NoSpecies(String),
    NoFoodEaters(String),
    RoundedTank,
    TankSize(u16),
    Predator { kind: String, size: u16 },
}

impl Constraint {
    pub fn subsumes(&self, other: &Constraint) -> bool {
        match (self, other) {
            (Quality(x), Quality(y)) => x > y,
            (NeedsLight(x), NeedsLight(y)) => x > y,
            (TankSize(x), TankSize(y)) => x > y,
            _ => false,
        }
    }
}
