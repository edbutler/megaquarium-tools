// pattern: Functional Core

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
    Communal(u8),
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
            (Cohabitation(Cohabitation::PairsOnly), _) => write!(f, "{} must only be a multiple of two", s),
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
            (Communal(others), _) => write!(f, "{} is communal and requires at least {} other species", s, others),
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
        Cohabitation(Cohabitation::PairsOnly) => {
            let count = exhibit.animals.iter().filter(|a| std::ptr::eq(anim.species, a.species)).count();
            simple(count % 2 == 0)
        }
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
            simple(exhibit.environment.size >= 2 * sum_size)
        }
        Predator { prey, size } => {
            let can_eat = |a: &&AnimalRef| a.species.prey_type == *prey && a.size_for_predation() <= *size;
            if_conflict(exhibit.animals.iter().find(can_eat))
        }
        Communal(others) => simple(count_distinct_by(exhibit.animals, |a| &a.species.id) > (*others as usize)),
    }
}

fn count_distinct_by<T, U: Ord, F: Fn(&T) -> U>(list: &[T], f: F) -> usize {
    let mut arr: Vec<_> = list.iter().map(f).collect();
    arr.sort();
    arr.dedup();
    arr.len()
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::animal::test::test_species;
    use crate::animal::*;
    use crate::tank::test::*;
    use crate::tank::*;

    static EMPTY_ANIMALS: &[AnimalRef<'static>] = &[];

    fn make_animal(species: &Species) -> AnimalRef<'_> {
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

    fn conflict_violation(animal: &AnimalRef, constraint: Constraint, conflicting: &AnimalRef) -> Violation {
        Violation {
            animal: animal.to_animal(),
            conflicting: Some(conflicting.to_animal()),
            constraint,
        }
    }

    fn make_animal_with_growth(species: &Species, growth: Growth) -> AnimalRef<'_> {
        AnimalRef { species, id: 0, growth }
    }

    fn species_with_genus(id: &str, genus: &str) -> Species {
        let mut s = test_species(id);
        s.genus = genus.to_string();
        s
    }

    fn species_with_fighting(id: &str, fighting: Option<Fighting>) -> Species {
        let mut s = test_species(id);
        s.fighting = fighting;
        s
    }

    fn species_with_nibbling(id: &str, nibbling: Option<Nibbling>) -> Species {
        let mut s = test_species(id);
        s.nibbling = nibbling;
        s
    }

    fn species_with_diet(id: &str, food: &str) -> Species {
        let mut s = test_species(id);
        s.diet = Diet::Food {
            food: food.to_string(),
            period: 1,
        };
        s
    }

    fn species_with_size(id: &str, final_size: u16, armored: bool) -> Species {
        let mut s = test_species(id);
        s.size.final_size = final_size;
        s.size.armored = armored;
        s
    }

    fn species_with_prey_type(id: &str, prey_type: PreyType) -> Species {
        let mut s = test_species(id);
        s.prey_type = prey_type;
        s
    }

    fn species_with_stages(id: &str, stages: Vec<Stage>, final_size: u16) -> Species {
        let mut s = test_species(id);
        s.size.stages = stages;
        s.size.final_size = final_size;
        s
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

    #[test]
    fn test_salinity() {
        let species = test_species("test");

        let salty_exhibit = simple_exhibit(Environment {
            salinity: tank::Salinity::Salty,
            ..test_environment()
        });

        let fresh_exhibit = simple_exhibit(Environment {
            salinity: tank::Salinity::Fresh,
            ..test_environment()
        });

        let animal = make_animal(&species);

        let salty_constraint = super::Salinity(tank::Salinity::Salty);
        let fresh_constraint = super::Salinity(tank::Salinity::Fresh);

        let salty_violation = simple_violation(&animal, salty_constraint);
        let fresh_violation = simple_violation(&animal, fresh_constraint);

        // Salty environment with salty constraint = pass
        assert_eq!(check_constraint(&salty_exhibit, &animal, &salty_constraint), None);
        // Fresh environment with salty constraint = fail
        assert_eq!(check_constraint(&fresh_exhibit, &animal, &salty_constraint), Some(salty_violation));
        // Fresh environment with fresh constraint = pass
        assert_eq!(check_constraint(&fresh_exhibit, &animal, &fresh_constraint), None);
        // Salty environment with fresh constraint = fail
        assert_eq!(check_constraint(&salty_exhibit, &animal, &fresh_constraint), Some(fresh_violation));
    }

    #[test]
    fn test_tank_size_and_interior() {
        let species = test_species("test");
        let animal = make_animal(&species);

        // TankSize tests
        let size_100_exhibit = simple_exhibit(Environment {
            size: 100,
            ..test_environment()
        });
        let size_150_exhibit = simple_exhibit(Environment {
            size: 150,
            ..test_environment()
        });
        let size_50_exhibit = simple_exhibit(Environment {
            size: 50,
            ..test_environment()
        });

        let size_constraint = TankSize(100);
        let size_violation = simple_violation(&animal, size_constraint);

        // Exact match = pass
        assert_eq!(check_constraint(&size_100_exhibit, &animal, &size_constraint), None);
        // Larger tank = pass
        assert_eq!(check_constraint(&size_150_exhibit, &animal, &size_constraint), None);
        // Smaller tank = fail
        assert_eq!(check_constraint(&size_50_exhibit, &animal, &size_constraint), Some(size_violation));

        // Interior tests
        let rounded_exhibit = simple_exhibit(Environment {
            interior: Some(tank::Interior::Rounded),
            ..test_environment()
        });
        let kreisel_exhibit = simple_exhibit(Environment {
            interior: Some(tank::Interior::Kreisel),
            ..test_environment()
        });
        let no_interior_exhibit = simple_exhibit(Environment {
            interior: None,
            ..test_environment()
        });

        let rounded_constraint = Interior(tank::Interior::Rounded);
        let rounded_violation = simple_violation(&animal, rounded_constraint);

        // Rounded interior with rounded constraint = pass
        assert_eq!(check_constraint(&rounded_exhibit, &animal, &rounded_constraint), None);
        // Kreisel interior with rounded constraint = fail
        assert_eq!(
            check_constraint(&kreisel_exhibit, &animal, &rounded_constraint),
            Some(rounded_violation.clone())
        );
        // No interior with rounded constraint = fail
        assert_eq!(
            check_constraint(&no_interior_exhibit, &animal, &rounded_constraint),
            Some(rounded_violation)
        );
    }

    #[test]
    fn test_bully_and_nibbler() {
        let wimp = species_with_fighting("wimp", Some(Fighting::Wimp));
        let bully = species_with_fighting("bully", Some(Fighting::Bully));
        let normal = test_species("normal");

        let wimp_animal = make_animal(&wimp);
        let bully_animal = make_animal(&bully);
        let normal_animal = make_animal(&normal);

        // NoBully tests - wimp requires no bullies
        let no_bully_constraint = NoBully;

        // Exhibit with just normal fish - no violation
        let normal_animals = [normal_animal.clone()];
        let normal_exhibit = ExhibitSpec {
            animals: &normal_animals,
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&normal_exhibit, &wimp_animal, &no_bully_constraint), None);

        // Exhibit with a bully - violation
        let bully_animals = [bully_animal.clone()];
        let bully_exhibit = ExhibitSpec {
            animals: &bully_animals,
            environment: test_environment(),
        };
        let bully_violation = conflict_violation(&wimp_animal, no_bully_constraint, &bully_animal);
        assert_eq!(
            check_constraint(&bully_exhibit, &wimp_animal, &no_bully_constraint),
            Some(bully_violation)
        );

        // NoNibbler tests
        let nibbleable = species_with_nibbling("nibbleable", Some(Nibbling::Nibbleable));
        let nibbler = species_with_nibbling("nibbler", Some(Nibbling::Nibbler));

        let nibbleable_animal = make_animal(&nibbleable);
        let nibbler_animal = make_animal(&nibbler);

        let no_nibbler_constraint = NoNibbler;

        // Exhibit with just normal fish - no violation
        assert_eq!(check_constraint(&normal_exhibit, &nibbleable_animal, &no_nibbler_constraint), None);

        // Exhibit with a nibbler - violation
        let nibbler_animals = [nibbler_animal.clone()];
        let nibbler_exhibit = ExhibitSpec {
            animals: &nibbler_animals,
            environment: test_environment(),
        };
        let nibbler_violation = conflict_violation(&nibbleable_animal, no_nibbler_constraint, &nibbler_animal);
        assert_eq!(
            check_constraint(&nibbler_exhibit, &nibbleable_animal, &no_nibbler_constraint),
            Some(nibbler_violation)
        );
    }

    #[test]
    fn test_lighting() {
        let species = test_species("test");
        let animal = make_animal(&species);

        // Dislikes light tests
        let no_light_exhibit = simple_exhibit(Environment {
            light: Some(0),
            ..test_environment()
        });
        let some_light_exhibit = simple_exhibit(Environment {
            light: Some(5),
            ..test_environment()
        });
        let none_light_exhibit = simple_exhibit(Environment {
            light: None,
            ..test_environment()
        });

        let dislikes_constraint = Lighting(Need::Dislikes);
        let dislikes_violation = simple_violation(&animal, dislikes_constraint);

        // No light = pass
        assert_eq!(check_constraint(&no_light_exhibit, &animal, &dislikes_constraint), None);
        // Some light = fail
        assert_eq!(
            check_constraint(&some_light_exhibit, &animal, &dislikes_constraint),
            Some(dislikes_violation.clone())
        );
        // None light = fail
        assert_eq!(
            check_constraint(&none_light_exhibit, &animal, &dislikes_constraint),
            Some(dislikes_violation)
        );

        // Loves light tests
        let light_5_exhibit = simple_exhibit(Environment {
            light: Some(5),
            ..test_environment()
        });
        let light_10_exhibit = simple_exhibit(Environment {
            light: Some(10),
            ..test_environment()
        });
        let light_3_exhibit = simple_exhibit(Environment {
            light: Some(3),
            ..test_environment()
        });

        let loves_5_constraint = Lighting(Need::Loves(5));
        let loves_5_violation = simple_violation(&animal, loves_5_constraint);

        // Light = 5, needs 5 = pass
        assert_eq!(check_constraint(&light_5_exhibit, &animal, &loves_5_constraint), None);
        // Light = 10, needs 5 = pass
        assert_eq!(check_constraint(&light_10_exhibit, &animal, &loves_5_constraint), None);
        // Light = 3, needs 5 = fail
        assert_eq!(
            check_constraint(&light_3_exhibit, &animal, &loves_5_constraint),
            Some(loves_5_violation.clone())
        );
        // Light = None, needs 5 = fail
        assert_eq!(
            check_constraint(&none_light_exhibit, &animal, &loves_5_constraint),
            Some(loves_5_violation)
        );
    }

    #[test]
    fn test_communal() {
        let species_a = test_species("species_a");
        let species_b = test_species("species_b");
        let species_c = test_species("species_c");

        let animal_a = make_animal(&species_a);
        let animal_b = make_animal(&species_b);
        let animal_c = make_animal(&species_c);

        let communal_2_constraint = Communal(2);
        let communal_2_violation = simple_violation(&animal_a, communal_2_constraint);

        // 3 distinct species, needs 2 others = pass (3 > 2)
        let three_species = [animal_a.clone(), animal_b.clone(), animal_c.clone()];
        let three_exhibit = ExhibitSpec {
            animals: &three_species,
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&three_exhibit, &animal_a, &communal_2_constraint), None);

        // 2 distinct species, needs 2 others = fail (2 not > 2)
        let two_species = [animal_a.clone(), animal_b.clone()];
        let two_exhibit = ExhibitSpec {
            animals: &two_species,
            environment: test_environment(),
        };
        assert_eq!(
            check_constraint(&two_exhibit, &animal_a, &communal_2_constraint),
            Some(communal_2_violation.clone())
        );

        // 1 distinct species, needs 2 others = fail
        let one_species = [animal_a.clone()];
        let one_exhibit = ExhibitSpec {
            animals: &one_species,
            environment: test_environment(),
        };
        assert_eq!(
            check_constraint(&one_exhibit, &animal_a, &communal_2_constraint),
            Some(communal_2_violation)
        );

        // Communal(1) with 2 species = pass
        let communal_1_constraint = Communal(1);
        assert_eq!(check_constraint(&two_exhibit, &animal_a, &communal_1_constraint), None);
    }

    #[test]
    fn test_shoaler() {
        let species = test_species("shoaler");
        let animal = make_animal(&species);

        let env = test_environment();

        // Shoaling config: needs 3, no exceptions
        let strict_shoaling = Shoaling {
            count: 3,
            one_ok: false,
            two_ok: false,
        };
        let strict_constraint = Shoaler(strict_shoaling);

        // Count = 3, needs 3 = pass (uses species ptr comparison)
        let three_animals = [animal.clone(), animal.clone(), animal.clone()];
        let three_exhibit = ExhibitSpec {
            animals: &three_animals,
            environment: env.clone(),
        };
        assert_eq!(check_constraint(&three_exhibit, &three_animals[0], &strict_constraint), None);

        // Count = 2, needs 3 = fail
        let two_animals = [animal.clone(), animal.clone()];
        let two_exhibit = ExhibitSpec {
            animals: &two_animals,
            environment: env.clone(),
        };
        let two_violation = simple_violation(&two_animals[0], strict_constraint);
        assert_eq!(
            check_constraint(&two_exhibit, &two_animals[0], &strict_constraint),
            Some(two_violation)
        );

        // Count = 5, needs 3 = pass
        let five_animals = [animal.clone(), animal.clone(), animal.clone(), animal.clone(), animal.clone()];
        let five_exhibit = ExhibitSpec {
            animals: &five_animals,
            environment: env.clone(),
        };
        assert_eq!(check_constraint(&five_exhibit, &five_animals[0], &strict_constraint), None);

        // Shoaling config: needs 3, one_ok = true
        let one_ok_shoaling = Shoaling {
            count: 3,
            one_ok: true,
            two_ok: false,
        };
        let one_ok_constraint = Shoaler(one_ok_shoaling);

        // Count = 1, one_ok = true = pass
        let one_animal = [animal.clone()];
        let one_exhibit = ExhibitSpec {
            animals: &one_animal,
            environment: env.clone(),
        };
        assert_eq!(check_constraint(&one_exhibit, &one_animal[0], &one_ok_constraint), None);

        // Count = 2, one_ok = true but two_ok = false = fail
        let one_ok_violation = simple_violation(&two_animals[0], one_ok_constraint);
        assert_eq!(
            check_constraint(&two_exhibit, &two_animals[0], &one_ok_constraint),
            Some(one_ok_violation)
        );

        // Shoaling config: needs 3, two_ok = true
        let two_ok_shoaling = Shoaling {
            count: 3,
            one_ok: false,
            two_ok: true,
        };
        let two_ok_constraint = Shoaler(two_ok_shoaling);

        // Count = 2, two_ok = true = pass
        assert_eq!(check_constraint(&two_exhibit, &two_animals[0], &two_ok_constraint), None);
    }

    #[test]
    fn test_cohabitation() {
        // OnlyCongeners - can only live with same genus
        let genus_a_1 = species_with_genus("species_a1", "genus_a");
        let genus_a_2 = species_with_genus("species_a2", "genus_a");
        let genus_b = species_with_genus("species_b", "genus_b");

        let animal_a1 = make_animal(&genus_a_1);
        let animal_a2 = make_animal(&genus_a_2);
        let animal_b = make_animal(&genus_b);

        let only_congeners = Cohabitation::OnlyCongeners;
        let only_congeners_constraint = super::Cohabitation(only_congeners);

        // Same genus only = pass (use references from array to avoid ptr_eq issues)
        let same_genus = [animal_a1.clone(), animal_a2.clone()];
        let same_genus_exhibit = ExhibitSpec {
            animals: &same_genus,
            environment: test_environment(),
        };
        assert_eq!(
            check_constraint(&same_genus_exhibit, &same_genus[0], &only_congeners_constraint),
            None
        );

        // Different genus present = fail
        let diff_genus = [animal_a1.clone(), animal_b.clone()];
        let diff_genus_exhibit = ExhibitSpec {
            animals: &diff_genus,
            environment: test_environment(),
        };
        let only_congeners_violation = conflict_violation(&diff_genus[0], only_congeners_constraint, &diff_genus[1]);
        assert_eq!(
            check_constraint(&diff_genus_exhibit, &diff_genus[0], &only_congeners_constraint),
            Some(only_congeners_violation)
        );

        // NoCongeners - cannot live with same genus
        let no_congeners = Cohabitation::NoCongeners;
        let no_congeners_constraint = super::Cohabitation(no_congeners);

        // Alone = pass
        let alone = [animal_a1.clone()];
        let alone_exhibit = ExhibitSpec {
            animals: &alone,
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&alone_exhibit, &alone[0], &no_congeners_constraint), None);

        // Different genus = pass
        assert_eq!(
            check_constraint(&diff_genus_exhibit, &diff_genus[0], &no_congeners_constraint),
            None
        );

        // Same genus (different species) = fail
        let no_congeners_violation = conflict_violation(&same_genus[0], no_congeners_constraint, &same_genus[1]);
        assert_eq!(
            check_constraint(&same_genus_exhibit, &same_genus[0], &no_congeners_constraint),
            Some(no_congeners_violation)
        );

        // NoConspecifics - cannot live with same species
        let no_conspecifics = Cohabitation::NoConspecifics;
        let no_conspecifics_constraint = super::Cohabitation(no_conspecifics);

        // Alone = pass
        assert_eq!(check_constraint(&alone_exhibit, &alone[0], &no_conspecifics_constraint), None);

        // Multiple of same species = fail (uses species ptr comparison, not animal ptr)
        let same_species = [animal_a1.clone(), animal_a1.clone()];
        let same_species_exhibit = ExhibitSpec {
            animals: &same_species,
            environment: test_environment(),
        };
        let no_conspecifics_violation = simple_violation(&same_species[0], no_conspecifics_constraint);
        assert_eq!(
            check_constraint(&same_species_exhibit, &same_species[0], &no_conspecifics_constraint),
            Some(no_conspecifics_violation)
        );

        // PairsOnly - must be even count (uses species ptr comparison)
        let pairs_only = Cohabitation::PairsOnly;
        let pairs_only_constraint = super::Cohabitation(pairs_only);

        // 2 fish = pass
        assert_eq!(
            check_constraint(&same_species_exhibit, &same_species[0], &pairs_only_constraint),
            None
        );

        // 3 fish = fail
        let three_same = [animal_a1.clone(), animal_a1.clone(), animal_a1.clone()];
        let three_same_exhibit = ExhibitSpec {
            animals: &three_same,
            environment: test_environment(),
        };
        let three_violation = simple_violation(&three_same[0], pairs_only_constraint);
        assert_eq!(
            check_constraint(&three_same_exhibit, &three_same[0], &pairs_only_constraint),
            Some(three_violation)
        );

        // NoFoodCompetitors - cannot share food type
        let eats_flakes = species_with_diet("flakes_eater", "flakes");
        let eats_pellets = species_with_diet("pellets_eater", "pellets");
        let also_eats_flakes = species_with_diet("also_flakes", "flakes");

        let flakes_animal = make_animal(&eats_flakes);
        let pellets_animal = make_animal(&eats_pellets);
        let also_flakes_animal = make_animal(&also_eats_flakes);

        let no_food_competitors = Cohabitation::NoFoodCompetitors;
        let no_food_competitors_constraint = super::Cohabitation(no_food_competitors);

        // Different food = pass
        let diff_food = [flakes_animal.clone(), pellets_animal.clone()];
        let diff_food_exhibit = ExhibitSpec {
            animals: &diff_food,
            environment: test_environment(),
        };
        assert_eq!(
            check_constraint(&diff_food_exhibit, &diff_food[0], &no_food_competitors_constraint),
            None
        );

        // Same food = fail
        let same_food = [flakes_animal.clone(), also_flakes_animal.clone()];
        let same_food_exhibit = ExhibitSpec {
            animals: &same_food,
            environment: test_environment(),
        };
        let food_violation = conflict_violation(&same_food[0], no_food_competitors_constraint, &same_food[1]);
        assert_eq!(
            check_constraint(&same_food_exhibit, &same_food[0], &no_food_competitors_constraint),
            Some(food_violation)
        );

        // Non-eater with food eater = pass (scavenger doesn't compete)
        let mut scavenger_species = test_species("scavenger");
        scavenger_species.diet = Diet::Scavenger;
        let scavenger_animal = make_animal(&scavenger_species);

        let mixed = [flakes_animal.clone(), scavenger_animal.clone()];
        let mixed_exhibit = ExhibitSpec {
            animals: &mixed,
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&mixed_exhibit, &mixed[0], &no_food_competitors_constraint), None);
    }

    #[test]
    fn test_territorial() {
        // Territorial species need tank size >= 2 * sum of their sizes
        let mut territorial_species = species_with_size("territorial", 10, false);
        territorial_species.habitat.territorial = true;
        let animal = make_animal(&territorial_species);

        let territorial_constraint = Territorial;

        // 2 fish of size 10 each, sum = 20, need tank >= 40 (uses species ptr comparison)
        let two_animals = [animal.clone(), animal.clone()];

        // Tank size 40 = exactly 50% = pass
        let exact_exhibit = ExhibitSpec {
            animals: &two_animals,
            environment: Environment {
                size: 40,
                ..test_environment()
            },
        };
        assert_eq!(check_constraint(&exact_exhibit, &two_animals[0], &territorial_constraint), None);

        // Tank size 50 = under 50% = pass
        let under_exhibit = ExhibitSpec {
            animals: &two_animals,
            environment: Environment {
                size: 50,
                ..test_environment()
            },
        };
        assert_eq!(check_constraint(&under_exhibit, &two_animals[0], &territorial_constraint), None);

        // 3 fish of size 10 each, sum = 30, need tank >= 60
        let three_animals = [animal.clone(), animal.clone(), animal.clone()];
        let over_exhibit = ExhibitSpec {
            animals: &three_animals,
            environment: Environment {
                size: 40,
                ..test_environment()
            },
        };
        let territorial_violation = simple_violation(&three_animals[0], territorial_constraint);
        assert_eq!(
            check_constraint(&over_exhibit, &three_animals[0], &territorial_constraint),
            Some(territorial_violation)
        );

        // Single fish of size 10, need tank >= 20
        let one_animal = [animal.clone()];
        let single_exhibit = ExhibitSpec {
            animals: &one_animal,
            environment: Environment {
                size: 20,
                ..test_environment()
            },
        };
        assert_eq!(check_constraint(&single_exhibit, &one_animal[0], &territorial_constraint), None);
    }

    #[test]
    fn test_predator() {
        // Predator with prey type Fish, can eat fish up to size 10
        let predator_constraint = Predator {
            prey: PreyType::Fish,
            size: 10,
        };

        let predator_species = test_species("predator");
        let test_animal = make_animal(&predator_species);

        // Wrong prey type - Crustacean won't be eaten by Fish predator
        let crustacean = species_with_prey_type("crab", PreyType::Crustacean);
        let crustacean_animal = make_animal(&crustacean);
        let crustacean_exhibit = ExhibitSpec {
            animals: &[crustacean_animal.clone()],
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&crustacean_exhibit, &test_animal, &predator_constraint), None);

        // Right type but too big - size 15 fish won't be eaten (15 > 10)
        let big_fish = species_with_size("big_fish", 15, false);
        let mut big_fish_species = big_fish;
        big_fish_species.prey_type = PreyType::Fish;
        let big_fish_animal = make_animal(&big_fish_species);
        let big_fish_exhibit = ExhibitSpec {
            animals: &[big_fish_animal.clone()],
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&big_fish_exhibit, &test_animal, &predator_constraint), None);

        // Right type and small enough - size 5 fish will be eaten (5 <= 10)
        let small_fish = species_with_size("small_fish", 5, false);
        let mut small_fish_species = small_fish;
        small_fish_species.prey_type = PreyType::Fish;
        let small_fish_animal = make_animal(&small_fish_species);
        let small_fish_exhibit = ExhibitSpec {
            animals: &[small_fish_animal.clone()],
            environment: test_environment(),
        };
        let small_fish_violation = conflict_violation(&test_animal, predator_constraint, &small_fish_animal);
        assert_eq!(
            check_constraint(&small_fish_exhibit, &test_animal, &predator_constraint),
            Some(small_fish_violation)
        );

        // Boundary case - size exactly equals predator limit (10 <= 10) - WILL be eaten
        let boundary_fish = species_with_size("boundary_fish", 10, false);
        let mut boundary_species = boundary_fish;
        boundary_species.prey_type = PreyType::Fish;
        let boundary_animal = make_animal(&boundary_species);
        let boundary_exhibit = ExhibitSpec {
            animals: &[boundary_animal.clone()],
            environment: test_environment(),
        };
        let boundary_violation = conflict_violation(&test_animal, predator_constraint, &boundary_animal);
        assert_eq!(
            check_constraint(&boundary_exhibit, &test_animal, &predator_constraint),
            Some(boundary_violation)
        );

        // Armored fish doubles effective size - size 8 armored = 16 effective, won't be eaten (16 > 10)
        let armored_fish = species_with_size("armored", 8, true);
        let mut armored_species = armored_fish;
        armored_species.prey_type = PreyType::Fish;
        let armored_animal = make_animal(&armored_species);
        let armored_exhibit = ExhibitSpec {
            animals: &[armored_animal.clone()],
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&armored_exhibit, &test_animal, &predator_constraint), None);

        // Armored but predator can eat up to 20 - size 8 armored = 16 effective, will be eaten (16 <= 20)
        let bigger_predator_constraint = Predator {
            prey: PreyType::Fish,
            size: 20,
        };
        let armored_violation = conflict_violation(&test_animal, bigger_predator_constraint, &armored_animal);
        assert_eq!(
            check_constraint(&armored_exhibit, &test_animal, &bigger_predator_constraint),
            Some(armored_violation)
        );

        // Growth stage tests - small fish at different growth stages
        let staged_fish = species_with_stages(
            "staged",
            vec![
                Stage { size: 2, duration: 10 }, // egg
                Stage { size: 4, duration: 20 }, // fry
            ],
            15, // adult size
        );
        let mut staged_species = staged_fish;
        staged_species.prey_type = PreyType::Fish;

        // Egg stage (size 2) will be eaten
        let egg_animal = make_animal_with_growth(&staged_species, Growth::Growing { stage: 0, growth: 0 });
        let egg_exhibit = ExhibitSpec {
            animals: &[egg_animal.clone()],
            environment: test_environment(),
        };
        let egg_violation = conflict_violation(&test_animal, predator_constraint, &egg_animal);
        assert_eq!(
            check_constraint(&egg_exhibit, &test_animal, &predator_constraint),
            Some(egg_violation)
        );

        // Fry stage (size 4) will be eaten
        let fry_animal = make_animal_with_growth(&staged_species, Growth::Growing { stage: 1, growth: 0 });
        let fry_exhibit = ExhibitSpec {
            animals: &[fry_animal.clone()],
            environment: test_environment(),
        };
        let fry_violation = conflict_violation(&test_animal, predator_constraint, &fry_animal);
        assert_eq!(
            check_constraint(&fry_exhibit, &test_animal, &predator_constraint),
            Some(fry_violation)
        );

        // Adult stage (size 15) won't be eaten (15 > 10)
        let adult_animal = make_animal_with_growth(&staged_species, Growth::Final);
        let adult_exhibit = ExhibitSpec {
            animals: &[adult_animal.clone()],
            environment: test_environment(),
        };
        assert_eq!(check_constraint(&adult_exhibit, &test_animal, &predator_constraint), None);
    }
}
