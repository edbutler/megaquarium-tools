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

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Temperature(t) => write!(f, "(temperature {})", t),
            Quality(q) => write!(f, "(quality {})", q),
            NeedsFood { kind, daily_amount } => write!(f, "(eats {} {})", kind, daily_amount),
            Scavenger => write!(f, "(scavenger)"),
            Shoaler(s) => write!(f, "(shoaler {})", s),
            IsBully => write!(f, "(bully)"),
            NoBully => write!(f, "(wimp)"),
            NoLight => write!(f, "(no-light)"),
            NeedsLight(l) => write!(f, "(light {})", l),
            OnlyGenus(g) => write!(f, "(only-genus {})", g),
            NoGenus(g) => write!(f, "(no-genus {})", g),
            NoSpecies(s) => write!(f, "(no-species {})", s),
            NoFoodEaters(e) => write!(f, "(no-food-eaters {})", e),
            RoundedTank => write!(f, "(rounded-tank)"),
            TankSize(s) => write!(f, "(tank-size {})", s),
            Predator { kind, size } => write!(f, "(predator {} {})", kind, size),
        }
    }
}
