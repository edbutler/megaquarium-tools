use crate::{
    animal::{self, Cohabitation, Diet, PreyType, Need},
    tank,
};
use Constraint::*;

pub struct RuleOptions {
    pub assume_all_fish_fully_grown: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Temperature(tank::Temperature),
    Quality(u8),
    Shoaler(u8),
    NoBully,
    Lighting(Need),
    Cohabitation(Cohabitation),
    TankType(animal::TankType),
    TankSize(u16),
    Predator { prey: PreyType, size: u16 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesSpec<'a> {
    pub species: &'a animal::Species,
    pub count: u16,
}

pub struct ExhibitSpec<'a> {
    pub options: RuleOptions,
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
            (Lighting(Need::Dislikes), None) => write!(f, "{} requires no light", s),
            (Lighting(Need::Dislikes), Some(o)) => write!(f, "{} requires no light but {} needs light", s, o),
            (Lighting(Need::Loves(l)), _) => write!(f, "{} requires at least {} light", s, l),
            (Cohabitation(Cohabitation::OnlyCongeners), Some(o)) => {
                write!(f, "{} requires congeners but there is {}", s, o)
            }
            (Cohabitation(Cohabitation::NoCongeners), Some(o)) => {
                if s == o {
                    write!(f, "{} cannot be with congeners but there are multiple {}", s, o)
                } else {
                    write!(f, "{} cannot be with congeners but there is {}", s, o)
                }
            }
            (Cohabitation(Cohabitation::NoConspecifics), _) => {
                write!(f, "{} cannot be with its own species but there are multiple", s)
            }
            (Cohabitation(Cohabitation::NoFoodCompetitors), Some(o)) => {
                write!(f, "{} will compete for food with {}", s, o)
            }
            (RoundedTank, _) => write!(f, "{} requies a rounded tank", s),
            (Predator { prey: _, size: _ }, Some(o)) => write!(f, "{} will eat {}", s, o),
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
            exhibit.animals.iter().find(|a| a.species.habitat.temperature != *t),
        ),
        Quality(q) => simple(*q <= exhibit.tank.environment.quality),
        Shoaler(c) => simple(s.count >= (*c as u16)),
        NoBully => if_conflict(exhibit.animals.iter().find(|a| a.species.is_bully())),
        Lighting(Need::Dislikes) => with_conflict(
            exhibit.tank.lighting == Some(0),
            exhibit.animals.iter().find(|a| a.species.needs_light()),
        ),
        Lighting(Need::Loves(l)) => simple(if let Some(x) = exhibit.tank.lighting {
            x >= *l
        } else {
            false
        }),
        Cohabitation(Cohabitation::OnlyCongeners) => {
            if_conflict(exhibit.animals.iter().find(|a| s.species.genus != a.species.genus))
        }
        Cohabitation(Cohabitation::NoCongeners) => if_conflict(exhibit.animals.iter().find(|a| {
            if std::ptr::eq(s, *a) {
                s.count > 1
            } else {
                s.species.genus == a.species.genus
            }
        })),
        Cohabitation(Cohabitation::NoConspecifics) => simple(s.count == 1),
        Cohabitation(Cohabitation::NoFoodCompetitors) => match &s.species.diet {
            Diet::Food {
                food: myfood,
                period: _,
            } => if_conflict(exhibit.animals.iter().find(|a| {
                !std::ptr::eq(s, *a)
                    && match &a.species.diet {
                        Diet::Food { food, period: _ } => myfood == food,
                        _ => false,
                    }
            })),
            _ => None,
        },
        RoundedTank => simple(exhibit.tank.rounded),
        TankSize(s) => simple(exhibit.tank.size >= *s),
        Predator { prey, size } => if_conflict(
            exhibit
                .animals
                .iter()
                .find(|a| a.species.prey_type == *prey && size_for_predation(&exhibit.options, a) <= *size),
        ),
    }
}

fn size_for_predation(options: &RuleOptions, spec: &SpeciesSpec) -> u16 {
    if options.assume_all_fish_fully_grown {
        spec.species.maximum_size()
    } else {
        spec.species.minimum_size()
    }
}
