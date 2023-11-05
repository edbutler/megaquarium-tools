use std::fmt::Display;

use crate::animal::*;
use crate::aquarium::*;
use crate::data::{self, GameData};
use crate::rules::*;
use crate::sexpr_format::*;
use crate::tank::*;
use crate::util::*;

pub struct CheckArgs<'a> {
    pub species: &'a [SpeciesCount],
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

pub struct CheckResult {
    pub violations: Vec<Violation>,
    pub food: Vec<FoodAmount>,
    pub minimum_viable_environment: Environment,
}

impl CheckResult {
    pub fn is_okay(&self) -> bool {
        self.violations.len() == 0
    }
}

pub struct ValidateArgs<'a> {
    pub aquarium: &'a AquariumRef<'a>,
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

pub fn check_for_viable_tank<'a>(data: &GameData, animals: &[AnimalRef]) -> CheckResult {
    let environment = minimum_viable_tank(&animals);
    let exhibit = ExhibitSpec { animals, environment };
    let violations = find_violations(&exhibit);
    let food = minimum_required_food(data, &exhibit.animals);

    CheckResult {
        violations,
        food,
        minimum_viable_environment: environment,
    }
}

pub fn print_violations(violations: &[Violation]) {
    let mut messages: Vec<_> = violations.iter().map(|v| v.to_string()).collect();
    messages.sort();
    messages.dedup();

    for v in messages {
        println!("- {}", v);
    }
}

pub fn print_check_result(args: &CheckArgs, result: &CheckResult) {
    println!("For contents:");
    for c in args.species {
        println!("- {}x {}", c.count, c.species);
    }

    if result.is_okay() {
        println!("\nThe minimum viable tank is:");
        if args.debug {
            println!("{:#?}", result.minimum_viable_environment);
        } else {
            println!(
                "{}",
                PrettyPrinted {
                    expr: result.minimum_viable_environment.to_sexp()
                }
            );
        }

        println!("\nWill require food (average per day):");
        for item in &result.food {
            println!("- {}x {}", item.count, item.food);
        }
    } else {
        println!("\nA valid tank is not possible:");
        print_violations(&result.violations);
    }
}

pub fn check_for_viable_aquarium(data: &data::GameData, args: &ValidateArgs) -> Result<()> {
    println!("Checking {} tanks...", args.aquarium.exhibits.len());

    let mut was_problem = false;

    for exhibit in &args.aquarium.exhibits {
        if exhibit.animals.len() == 0 {
            continue;
        }

        let min_tank = minimum_viable_tank(&exhibit.animals);

        println!("{}:", exhibit.name);
        // TODO this isn't quite right if some fish are not grown
        println!("- {}/{}, {}%", min_tank.size, exhibit.tank.volume(), min_tank.quality);

        for item in minimum_required_food(data, &exhibit.animals) {
            println!("- {}x {}", item.count, item.food);
        }

        let exhibit_spec = ExhibitSpec {
            animals: &exhibit.animals,
            environment: min_tank,
        };

        let violations = find_violations(&exhibit_spec);
        print_violations(&violations);
        was_problem = was_problem || violations.len() > 0;
    }

    if !was_problem {
        println!("No problems!");
    }

    Ok(())
}

pub fn try_expand_tank(data: &GameData, base: &ExhibitRef, expansion: &ExhibitSpec) -> CheckResult {
    let mut animals = base.animals.clone();
    animals.extend(expansion.animals);
    check_for_viable_tank(data, &animals)
}

pub fn print_environment_differences(old: &Environment, new: &Environment) {
    fn format_opt<T>(x: Option<T>) -> String
    where
        T: Display,
    {
        match x {
            Some(v) => format!("{}", v),
            None => "n/a".to_string(),
        }
    }

    fn compare<T>(name: &str, old: T, new: T)
    where
        T: Display + PartialOrd,
    {
        if old < new {
            println!("- {}: {} → {}", name, old, new);
        }
    }

    fn compare_opt<T>(name: &str, old: Option<T>, new: Option<T>)
    where
        T: Display + PartialOrd,
    {
        if old < new {
            println!("- {}: {} → {}", name, format_opt(old), format_opt(new));
        }
    }

    compare("size", old.size, new.size);
    compare("quality", old.quality, new.quality);
    compare_opt("plants", old.plants, new.plants);
    compare_opt("rocks", old.rocks, new.rocks);
    compare_opt("caves", old.caves, new.caves);
    compare_opt("light", old.light, new.light);
}

pub fn animals_from_counts<'a>(data: &'a GameData, args: &CheckArgs) -> Result<Vec<AnimalRef<'a>>> {
    let mut counter = 0;
    let capacity: u16 = args.species.iter().map(|c| c.count).sum();
    let mut result = Vec::with_capacity(capacity as usize);

    for c in args.species {
        let species = lookup(data, &c.species)?;

        for _ in 0..c.count {
            counter += 1;
            let growth = if args.assume_all_fish_fully_grown {
                Growth::Final
            } else {
                species.earliest_growth_stage()
            };
            result.push(AnimalRef {
                id: counter,
                species: species,
                growth,
            });
        }
    }

    Ok(result)
}

pub fn environment_for_exhibit(exhibit: &ExhibitRef) -> Environment {
    let mut result = minimum_viable_tank(&exhibit.animals);

    // have to correct size to the known tank size since some animals may not be grown
    result.size = exhibit.tank.volume();
    result.interior = exhibit.tank.model.interior;

    result
}

// Guess at the minimum viable tank for the given species.
// Still requires checking for constraint violations.
fn minimum_viable_tank(animals: &[AnimalRef<'_>]) -> Environment {
    if animals.len() == 0 {
        panic!("need to specify at least some animals");
    }

    let mut size = animals.iter().map(|a| a.species.maximum_size()).sum();
    size = std::cmp::max(size, animals.iter().map(|a| a.species.minimum_needed_tank_size()).max().unwrap());

    for a in animals {
        if a.species.habitat.territorial {
            let sum_size: u16 = animals
                .iter()
                .map(|o| {
                    if std::ptr::eq(o.species, a.species) {
                        o.species.maximum_size()
                    } else {
                        0
                    }
                })
                .sum();
            size = std::cmp::max(size, 2 * sum_size);
        }
    }

    let light = animals
        .iter()
        .filter_map(|s| match s.species.needs.light {
            Some(Need::Loves(x)) => Some(x),
            Some(Need::Dislikes) => Some(0),
            None => None,
        })
        .max();

    Environment {
        size,
        temperature: animals[0].species.habitat.temperature,
        salinity: animals.iter().find_map(|a| a.species.habitat.salinity).unwrap_or(Salinity::Salty),
        quality: animals.iter().map(|s| s.species.habitat.minimum_quality).max().unwrap(),
        plants: minimum_need(animals, |s| s.needs.plants),
        rocks: minimum_need(animals, |s| s.needs.rocks),
        caves: minimum_need(animals, |s| s.needs.caves.map(|x| Need::Loves(x))),
        light,
        interior: animals.iter().find_map(|s| s.species.habitat.interior),
    }
}

pub struct FoodAmount {
    pub food: String,
    pub count: u16,
}

fn minimum_required_food(data: &GameData, species: &[AnimalRef<'_>]) -> Vec<FoodAmount> {
    let diets: Vec<(&String, u16)> = species
        .iter()
        .filter_map(|s| match &s.species.diet {
            Diet::Food { food, period: _ } => Some((food, s.species.amount_food_eaten())),
            _ => None,
        })
        .collect();

    data.food
        .iter()
        .filter_map(|food| {
            let count = diets.iter().filter_map(|(x, c)| if food == *x { Some(c) } else { None }).sum();
            if count > 0 {
                Some(FoodAmount { food: food.clone(), count })
            } else {
                None
            }
        })
        .collect()
}

fn minimum_need<F: Fn(&Species) -> Option<Need>>(list: &[AnimalRef], f: F) -> Option<u16> {
    let foldfn = |acc: Option<u16>, a: &AnimalRef| -> Option<u16> {
        match f(a.species) {
            Some(Need::Dislikes) => Some(0),
            Some(Need::Loves(x)) => Some((x as u16) + acc.unwrap_or(0)),
            None => acc,
        }
    };

    list.iter().fold(None, foldfn)
}

fn lookup<'a>(data: &'a data::GameData, species: &str) -> Result<&'a Species> {
    let possible = data.species_search(species);

    if possible.len() == 0 {
        Err(error(format!("No matching species for '{}'", species)))
    } else if possible.len() > 1 {
        let list: Vec<&String> = possible.iter().map(|s| &s.id).collect();
        Err(error(format!("Ambiguous match for '{}': {:#?}", species, list)))
    } else {
        Ok(possible[0])
    }
}
