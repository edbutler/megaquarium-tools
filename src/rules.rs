use crate::{animal, tank};
use Constraint::*;

#[derive(Debug, Clone, PartialEq)]
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

pub struct SpeciesSpec<'a> {
    pub species: &'a animal::Species,
    pub count: u16,
}

pub struct ExhibitSpec<'a> {
    pub animals: &'a [SpeciesSpec<'a>],
    pub tank: tank::TankStatus,
}

pub struct Violation {
    pub species: String,
    pub constraint: Constraint,
    pub conflicting_species: Option<String>,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = &self.species;
        let o = self.conflicting_species.as_ref();

        match &self.constraint {
            Temperature(t) => write!(f, "{} requires {} tank", s, t),
            Quality(q) => write!(f, "{} requires at least quality {}", s, q),
            Shoaler(c) => write!(f, "{} is a shoaler and needs {} of its species", s, c),
            NoBully => write!(f, "{} will bully {}", o.unwrap(), s),
            NoLight => todo!("nolight"),
            NeedsLight(l) => todo!("needslight"),
            _ => todo!(),
        }
    }
}

pub fn find_violations(exhibit: &ExhibitSpec) -> Vec<Violation> {
    let mut result = Vec::new();

    for s in exhibit.animals {
        for c in s.species.constraints() {
            if let Some(v) = check_constraint(exhibit, s, &c) {
                result.push(v);
            }
        }
    }

    result
}

fn check_constraint<'a>(exhibit: &ExhibitSpec<'a>, s: &SpeciesSpec<'a>, constraint: &Constraint) -> Option<Violation> {
    let simple = |is_okay: bool| {
        if is_okay {
            None
        } else {
            Some(Violation {
                species: s.species.id.clone(),
                constraint: constraint.clone(),
                conflicting_species: None,
            })
        }
    };

    let conflict = |other: Option<&SpeciesSpec>| match other {
        None => None,
        Some(o) => Some(Violation {
            species: s.species.id.clone(),
            constraint: constraint.clone(),
            conflicting_species: Some(o.species.id.clone()),
        }),
    };

    match constraint {
        Temperature(t) => simple(*t == exhibit.tank.environment.temperature),
        Quality(q) => simple(*q <= exhibit.tank.environment.quality),
        Shoaler(c) => simple(s.count >= (*c as u16)),
        NoBully => conflict(exhibit.animals.iter().find(|a| a.species.is_bully())),
        NoLight => simple(exhibit.tank.lighting == 0),
        NeedsLight(l) => simple(exhibit.tank.lighting >= *l),
        //OnlyGenus(String),
        //NoGenus(String),
        //NoSpecies(String),
        //NoFoodEaters(String),
        RoundedTank => simple(exhibit.tank.rounded),
        TankSize(s) => simple(exhibit.tank.size >= *s),
        //Predator { kind: String, size: u16 },
        _ => None,
    }
}
