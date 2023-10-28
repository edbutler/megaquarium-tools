use crate::{
    animal::{Animal, Cohabitation, Diet, Need, PreyType},
    tank,
};
use Constraint::*;

pub struct RuleOptions {
    pub assume_all_fish_fully_grown: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constraint {
    Temperature(tank::Temperature),
    Quality(u8),
    Shoaler(u8),
    NoBully,
    Lighting(Need),
    Cohabitation(Cohabitation),
    Interior(tank::Interior),
    TankSize(u16),
    Predator { prey: PreyType, size: u16 },
}

pub struct ExhibitSpec<'a> {
    pub options: &'a RuleOptions,
    pub animals: &'a [Animal<'a>],
    pub tank: tank::Environment,
}

#[derive(Debug, Clone)]
pub struct Violation<'a> {
    pub species: &'a Animal<'a>,
    pub constraint: Constraint,
    pub conflicting: Option<&'a Animal<'a>>,
}

impl PartialEq for Violation<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.species, other.species)
            && self.constraint == other.constraint
            && match (self.conflicting, other.conflicting) {
                (None, None) => true,
                (Some(a), Some(b)) => std::ptr::eq(a.species, b.species),
                _ => false,
            }
    }
}

impl std::fmt::Display for Violation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = self.species.species;

        match (&self.constraint, &self.conflicting) {
            (Temperature(t), None) => write!(f, "{} requires {} tank", s.id, t),
            (Temperature(t), Some(o)) => write!(f, "{} requires {} tank but {} requires {}", s.id, t, o.species.id, t.other()),
            (Quality(q), _) => write!(f, "{} requires at least quality {}", s.id, q),
            (Shoaler(c), _) => write!(f, "{} is a shoaler and needs {} of its species", s.id, c),
            (NoBully, Some(o)) => write!(f, "{} will bully {}", o.species.id, s.id),
            (Lighting(Need::Dislikes), None) => write!(f, "{} requires no light", s.id),
            (Lighting(Need::Dislikes), Some(o)) => {
                write!(f, "{} requires no light but {} needs light", s.id, o.species.id)
            }
            (Lighting(Need::Loves(l)), _) => write!(f, "{} requires at least {} light", s.id, l),
            (Cohabitation(Cohabitation::OnlyCongeners), Some(o)) => {
                write!(f, "{} requires congeners but there is {}", s.id, o.species.id)
            }
            (Cohabitation(Cohabitation::NoCongeners), Some(o)) => {
                if std::ptr::eq(s, o.species) {
                    write!(f, "{} cannot be with congeners but there are multiple {}", s.id, o.species.id)
                } else {
                    write!(f, "{} cannot be with congeners but there is {}", s.id, o.species.id)
                }
            }
            (Cohabitation(Cohabitation::NoConspecifics), _) => {
                write!(f, "{} cannot be with its own species but there are multiple", s.id)
            }
            (Cohabitation(Cohabitation::NoFoodCompetitors), Some(o)) => {
                write!(f, "{} will compete for food with {}", s.id, o.species.id)
            }
            (Interior(tank::Interior::Rounded), _) => write!(f, "{} requies a rounded tank", s.id),
            (Interior(tank::Interior::Kreisel), _) => write!(f, "{} requies a kreisel tank", s.id),
            (Predator { prey: _, size }, Some(o)) => {
                if o.species.maximum_size() > *size {
                    write!(f, "{} will eat {} (though it will be fine if fully grown)", s.id, o.species.id)
                } else {
                    write!(f, "{} will eat {}", s.id, o.species.id)
                }
            }
            _ => todo!(),
        }
    }
}

pub fn find_violations<'a>(exhibit: &'a ExhibitSpec<'a>) -> Vec<Violation<'a>> {
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

fn check_constraint<'a>(exhibit: &'a ExhibitSpec<'a>, anim: &'a Animal<'a>, constraint: &Constraint) -> Option<Violation<'a>> {
    let simple = |is_okay: bool| {
        if is_okay {
            None
        } else {
            Some(Violation {
                species: anim,
                constraint: constraint.clone(),
                conflicting: None,
            })
        }
    };

    let if_conflict = |other: Option<&'a Animal<'a>>| match other {
        None => None,
        Some(o) => Some(Violation {
            species: anim,
            constraint: constraint.clone(),
            conflicting: Some(o),
        }),
    };

    let with_conflict = |is_okay: bool, conflict: Option<&'a Animal<'a>>| {
        if is_okay {
            None
        } else {
            Some(Violation {
                species: anim,
                constraint: constraint.clone(),
                conflicting: conflict,
            })
        }
    };

    match constraint {
        Temperature(t) => with_conflict(
            *t == exhibit.tank.temperature,
            exhibit.animals.iter().find(|a| a.species.habitat.temperature != *t),
        ),
        Quality(q) => simple(*q <= exhibit.tank.quality),
        Shoaler(c) => {
            let count = exhibit.animals.iter().filter(|a| std::ptr::eq(anim.species, a.species)).count();
            simple(count >= (*c as usize))
        }
        NoBully => if_conflict(exhibit.animals.iter().find(|a| a.species.is_bully())),
        Lighting(Need::Dislikes) => with_conflict(
            exhibit.tank.light == Some(0),
            exhibit.animals.iter().find(|a| a.species.needs_light()),
        ),
        Lighting(Need::Loves(l)) => simple(if let Some(x) = exhibit.tank.light { x >= *l } else { false }),
        Cohabitation(Cohabitation::OnlyCongeners) => if_conflict(exhibit.animals.iter().find(|a| anim.species.genus != a.species.genus)),
        Cohabitation(Cohabitation::NoCongeners) => if_conflict(
            exhibit
                .animals
                .iter()
                .find(|a| !std::ptr::eq(*a, anim) && anim.species.genus == a.species.genus),
        ),
        Cohabitation(Cohabitation::NoConspecifics) => simple(
            exhibit
                .animals
                .iter()
                .all(|a| std::ptr::eq(a, anim) || !std::ptr::eq(anim.species, a.species)),
        ),
        Cohabitation(Cohabitation::NoFoodCompetitors) => match &anim.species.diet {
            Diet::Food { food: myfood, period: _ } => if_conflict(exhibit.animals.iter().find(|a| {
                !std::ptr::eq(anim.species, a.species)
                    && match &a.species.diet {
                        Diet::Food { food, period: _ } => myfood == food,
                        _ => false,
                    }
            })),
            _ => None,
        },
        Interior(i) => simple(exhibit.tank.interior == Some(*i)),
        TankSize(s) => simple(exhibit.tank.size >= *s),
        Predator { prey, size } => if_conflict(exhibit.animals.iter().find(|a| a.species.prey_type == *prey && a.size() <= *size)),
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::animal::test::test_species;
    use crate::animal::*;
    use crate::tank::test::*;
    use crate::tank::*;

    static OPTIONS: RuleOptions = RuleOptions {
        assume_all_fish_fully_grown: false,
    };
    static EMPTY_ANIMALS: &[Animal<'static>] = &[];

    fn make_animal(species: &Species) -> Animal {
        Animal {
            species: &species,
            id: 0,
            growth: Growth::Final,
        }
    }

    fn simple_exhibit(environment: Environment) -> ExhibitSpec<'static> {
        ExhibitSpec {
            options: &OPTIONS,
            animals: &EMPTY_ANIMALS,
            tank: environment,
        }
    }

    fn simple_violation<'a>(animal: &'a Animal<'a>, constraint: Constraint) -> Violation<'a> {
        Violation {
            species: animal,
            conflicting: None,
            constraint,
        }
    }

    #[test]
    fn test_temperature() {
        let species = test_species("test");

        let warm_exhibit = simple_exhibit(Environment {
            temperature: tank::Temperature::Warm,
            ..test_environment()
        });

        let cold_exhibit = simple_exhibit(Environment {
            temperature: tank::Temperature::Cold,
            ..test_environment()
        });

        let animal = make_animal(&species);

        let warm_constraint = super::Temperature(tank::Temperature::Warm);
        let cold_constraint = super::Temperature(tank::Temperature::Cold);

        let warm_violation = simple_violation(&animal, warm_constraint);
        let cold_violation = simple_violation(&animal, cold_constraint);

        assert_eq!(check_constraint(&warm_exhibit, &animal, &warm_constraint), None);
        assert_eq!(check_constraint(&cold_exhibit, &animal, &warm_constraint), Some(warm_violation));
        assert_eq!(check_constraint(&cold_exhibit, &animal, &cold_constraint), None);
        assert_eq!(check_constraint(&warm_exhibit, &animal, &cold_constraint), Some(cold_violation));
    }

    #[test]
    fn test_quality() {
        let species = test_species("test");

        let q55_exhibit = simple_exhibit(Environment {
            quality: 55,
            ..test_environment()
        });

        let q64_exhibit = simple_exhibit(Environment {
            quality: 64,
            ..test_environment()
        });

        let q65_exhibit = simple_exhibit(Environment {
            quality: 65,
            ..test_environment()
        });

        let animal = make_animal(&species);

        let q60_constraint = Quality(60);
        let q65_constraint = Quality(65);

        let q60_violation = simple_violation(&animal, q60_constraint);
        let q65_violation = simple_violation(&animal, q65_constraint);

        assert_eq!(check_constraint(&q55_exhibit, &animal, &q60_constraint), Some(q60_violation));
        assert_eq!(check_constraint(&q55_exhibit, &animal, &q65_constraint), Some(q65_violation.clone()));
        assert_eq!(check_constraint(&q64_exhibit, &animal, &q60_constraint), None);
        assert_eq!(check_constraint(&q64_exhibit, &animal, &q65_constraint), Some(q65_violation.clone()));
        assert_eq!(check_constraint(&q65_exhibit, &animal, &q60_constraint), None);
        assert_eq!(check_constraint(&q65_exhibit, &animal, &q65_constraint), None);
    }
}
