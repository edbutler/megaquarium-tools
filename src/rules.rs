use crate::{
    animal::{Animal, AnimalRef, Cohabitation, Diet, Growth, Need, PreyType, Shoaling},
    tank,
};
use Constraint::*;

pub struct RuleOptions {
    pub assume_all_fish_fully_grown: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constraint {
    Temperature(tank::Temperature),
    Salinity(tank::Salinity),
    Quality(u8),
    Shoaler(Shoaling),
    NoBully,
    NoNibbler,
    Lighting(Need),
    Cohabitation(Cohabitation),
    Interior(tank::Interior),
    TankSize(u16),
    Territorial,
    Predator { prey: PreyType, size: u16 },
}

pub struct ExhibitSpec<'a> {
    pub animals: &'a [AnimalRef<'a>],
    pub environment: tank::Environment,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Violation {
    pub animal: Animal,
    pub constraint: Constraint,
    pub conflicting: Option<Animal>,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = &self.animal.species;

        match (&self.constraint, &self.conflicting) {
            (Temperature(t), None) => write!(f, "{} requires {} tank", s, t),
            (Temperature(t), Some(o)) => write!(f, "{} requires {} tank but {} requires {}", s, t, o.species, t.other()),
            (Salinity(x), None) => write!(f, "{} requires {} tank", s, x),
            (Salinity(x), Some(o)) => write!(f, "{} requires {} tank but {} requires {}", s, x, o.species, x.other()),
            (Quality(q), _) => write!(f, "{} requires at least quality {}", s, q),
            (Shoaler(c), _) => {
                let or1 = if c.one_ok { ", or 1" } else { "" };
                let or2 = if c.one_ok { ", or 2" } else { "" };
                write!(f, "{} is a shoaler and needs {} of its species{}{}", s, c.count, or1, or2)
            }
            (NoBully, Some(o)) => write!(f, "{} will bully {}", o.species, s),
            (NoNibbler, Some(o)) => write!(f, "{} will nibble {}", o.species, s),
            (Lighting(Need::Dislikes), None) => write!(f, "{} requires no light", s),
            (Lighting(Need::Dislikes), Some(o)) => {
                write!(f, "{} requires no light but {} needs light", s, o.species)
            }
            (Lighting(Need::Loves(l)), _) => write!(f, "{} requires at least {} light", s, l),
            (Cohabitation(Cohabitation::OnlyCongeners), Some(o)) => {
                write!(f, "{} requires congeners but there is {}", s, o.species)
            }
            (Cohabitation(Cohabitation::NoCongeners), Some(o)) => {
                if *s == o.species {
                    write!(f, "{} cannot be with congeners but there are multiple {}", s, o.species)
                } else {
                    write!(f, "{} cannot be with congeners but there is {}", s, o.species)
                }
            }
            (Cohabitation(Cohabitation::NoConspecifics), _) => {
                write!(f, "{} cannot be with its own species but there are multiple", s)
            }
            (Cohabitation(Cohabitation::NoFoodCompetitors), Some(o)) => {
                write!(f, "{} will compete for food with {}", s, o.species)
            }
            (Interior(tank::Interior::Rounded), _) => write!(f, "{} requies a rounded tank", s),
            (Interior(tank::Interior::Kreisel), _) => write!(f, "{} requies a kreisel tank", s),
            (Territorial, _) => write!(f, "{} is territorial, total size can only be 50% of tank size", s),
            (Predator { prey: _, size: _ }, Some(o)) => {
                if o.growth != Growth::Final {
                    // TODO need to determine this completely
                    write!(f, "{} will eat {} (though may be fine if fully grown)", s, o.species)
                } else {
                    write!(f, "{} will eat {}", s, o.species)
                }
            }
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

fn check_constraint<'a>(exhibit: &'a ExhibitSpec<'a>, anim: &'a AnimalRef<'a>, constraint: &Constraint) -> Option<Violation> {
    let simple = |is_okay: bool| {
        if is_okay {
            None
        } else {
            Some(Violation {
                animal: anim.to_animal(),
                constraint: constraint.clone(),
                conflicting: None,
            })
        }
    };

    let if_conflict = |other: Option<&'a AnimalRef<'a>>| match other {
        None => None,
        Some(o) => Some(Violation {
            animal: anim.to_animal(),
            constraint: constraint.clone(),
            conflicting: Some(o.to_animal()),
        }),
    };

    let with_conflict = |is_okay: bool, conflict: Option<&'a AnimalRef<'a>>| {
        if is_okay {
            None
        } else {
            Some(Violation {
                animal: anim.to_animal(),
                constraint: constraint.clone(),
                conflicting: conflict.map(|x| x.to_animal()),
            })
        }
    };

    match constraint {
        Temperature(t) => with_conflict(
            *t == exhibit.environment.temperature,
            exhibit.animals.iter().find(|a| a.species.habitat.temperature != *t),
        ),
        Salinity(s) => with_conflict(
            *s == exhibit.environment.salinity,
            exhibit
                .animals
                .iter()
                .find(|a| a.species.habitat.salinity.map_or(false, |x| x != *s)),
        ),
        Quality(q) => simple(*q <= exhibit.environment.quality),
        Shoaler(c) => {
            let count = exhibit.animals.iter().filter(|a| std::ptr::eq(anim.species, a.species)).count();
            let is_okay = (c.one_ok && count == 1) || (c.two_ok && count == 2) || (count >= (c.count as usize));
            simple(is_okay)
        }
        NoBully => if_conflict(exhibit.animals.iter().find(|a| a.species.is_bully())),
        NoNibbler => if_conflict(exhibit.animals.iter().find(|a| a.species.is_nibbler())),
        Lighting(Need::Dislikes) => with_conflict(
            exhibit.environment.light == Some(0),
            exhibit.animals.iter().find(|a| a.species.needs_light()),
        ),
        Lighting(Need::Loves(l)) => simple(if let Some(x) = exhibit.environment.light { x >= *l } else { false }),
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
        Interior(i) => simple(exhibit.environment.interior == Some(*i)),
        TankSize(s) => simple(exhibit.environment.size >= *s),
        Territorial => {
            // tank must be twice as big as sum of sizes of this species
            let sum_size: u16 = exhibit
                .animals
                .iter()
                .map(|a| {
                    if std::ptr::eq(anim.species, a.species) {
                        a.species.maximum_size()
                    } else {
                        0
                    }
                })
                .sum();
            println!("{}", sum_size);
            simple(exhibit.environment.size >= 2 * sum_size)
        }
        Predator { prey, size } => {
            let can_eat = |a: &&AnimalRef| a.species.prey_type == *prey && a.size_for_predation() <= *size;
            if_conflict(exhibit.animals.iter().find(can_eat))
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::animal::test::test_species;
    use crate::animal::*;
    use crate::tank::test::*;
    use crate::tank::*;

    static EMPTY_ANIMALS: &[AnimalRef<'static>] = &[];

    fn make_animal(species: &Species) -> AnimalRef {
        AnimalRef {
            species: &species,
            id: 0,
            growth: Growth::Final,
        }
    }

    fn simple_exhibit(environment: Environment) -> ExhibitSpec<'static> {
        ExhibitSpec {
            animals: &EMPTY_ANIMALS,
            environment,
        }
    }

    fn simple_violation(animal: &AnimalRef, constraint: Constraint) -> Violation {
        Violation {
            animal: animal.to_animal(),
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
        assert_eq!(
            check_constraint(&q55_exhibit, &animal, &q65_constraint),
            Some(q65_violation.clone())
        );
        assert_eq!(check_constraint(&q64_exhibit, &animal, &q60_constraint), None);
        assert_eq!(
            check_constraint(&q64_exhibit, &animal, &q65_constraint),
            Some(q65_violation.clone())
        );
        assert_eq!(check_constraint(&q65_exhibit, &animal, &q60_constraint), None);
        assert_eq!(check_constraint(&q65_exhibit, &animal, &q65_constraint), None);
    }
}
