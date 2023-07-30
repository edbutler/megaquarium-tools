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

        match (&self.constraint, &self.conflicting_species) {
            (Temperature(t), None) => write!(f, "{} requires {} tank", s, t),
            (Temperature(t), Some(o)) => write!(f, "{} requires {} tank but {} requires {}", s, t, o, t.other()),
            (Quality(q), _) => write!(f, "{} requires at least quality {}", s, q),
            (Shoaler(c), _) => write!(f, "{} is a shoaler and needs {} of its species", s, c),
            (NoBully, Some(o)) => write!(f, "{} will bully {}", o, s),
            (NoLight, Some(o)) => write!(f, "{} requires no light but {} needs light", s, o),
            (NeedsLight(l), _) => write!(f, "{} requires at least {} light", s, l),
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

    let if_conflict = |other: Option<&SpeciesSpec>| match other {
        None => None,
        Some(o) => Some(Violation {
            species: s.species.id.clone(),
            constraint: constraint.clone(),
            conflicting_species: Some(o.species.id.clone()),
        }),
    };

    let with_conflict = |is_okay: bool, conflict: Option<&SpeciesSpec>| {
        if is_okay {
            None
        } else {
            let conflicting_species = conflict.map(|s| s.species.id.clone());
            Some(Violation {
                species: s.species.id.clone(),
                constraint: constraint.clone(),
                conflicting_species,
            })
        }
    };

    match constraint {
        Temperature(t) => with_conflict(
            *t == exhibit.tank.environment.temperature,
            exhibit.animals.iter().find(|a| a.species.environment.temperature != *t),
        ),
        Quality(q) => simple(*q <= exhibit.tank.environment.quality),
        Shoaler(c) => simple(s.count >= (*c as u16)),
        NoBully => if_conflict(exhibit.animals.iter().find(|a| a.species.is_bully())),
        NoLight => with_conflict(
            exhibit.tank.lighting == 0,
            exhibit.animals.iter().find(|a| a.species.needs_light())
        ),
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
