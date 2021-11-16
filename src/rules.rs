use crate::tank::*;

pub enum Constraint {
    Temperature(Temperature),
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

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constraint::Temperature(t) => write!(f, "(temperature {})", t),
            Constraint::Quality(q) => write!(f, "(quality {})", q),
            Constraint::NeedsFood { kind, daily_amount } => write!(f, "(eats {} {})", kind, daily_amount),
            Constraint::Scavenger => write!(f, "(scavenger)"),
            Constraint::Shoaler(s) => write!(f, "(shoaler {})", s),
            Constraint::IsBully => write!(f, "(bully)"),
            Constraint::NoBully => write!(f, "(wimp)"),
            Constraint::NoLight => write!(f, "(no-light)"),
            Constraint::NeedsLight(l) => write!(f, "(light {})", l),
            Constraint::OnlyGenus(g) => write!(f, "(only-genus {})", g),
            Constraint::NoGenus(g) => write!(f, "(no-genus {})", g),
            Constraint::NoSpecies(s) => write!(f, "(no-species {})", s),
            Constraint::NoFoodEaters(e) => write!(f, "(no-food-eaters {})", e),
            Constraint::RoundedTank => write!(f, "(rounded-tank)"),
            Constraint::TankSize(s) => write!(f, "(tank-size {})", s),
            Constraint::Predator { kind, size } => write!(f, "(predator {} {})", kind, size),
        }
    }
}
