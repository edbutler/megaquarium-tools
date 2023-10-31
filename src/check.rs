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

pub struct ValidateArgs {
    pub aquarium: AquariumDesc,
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

pub fn check_for_viable_tank<'a>(data: &GameData, args: &CheckArgs, animals: &[AnimalRef]) -> Result<CheckResult> {
    let environment = minimum_viable_tank(&animals);
    let exhibit = ExhibitSpec { animals, environment };
    let violations = find_violations(&exhibit);
    let food = minimum_required_food(data, &exhibit.animals);

    Ok(CheckResult {
        violations,
        food,
        minimum_viable_environment: environment,
    })
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
        for v in &result.violations {
            println!("- {}", v);
        }
    }
}

pub fn check_for_viable_aquarium(data: &data::GameData, args: &ValidateArgs) -> Result<()> {
    println!("Checking {} tanks...", args.aquarium.exhibits.len());

    let options = RuleOptions {
        assume_all_fish_fully_grown: args.assume_all_fish_fully_grown,
    };

    let mut was_problem = false;

    for exhibit in &args.aquarium.exhibits {
        let mut animals = Vec::new();
        let mut counter = 0;

        for desc in &exhibit.animals {
            match desc {
                AnimalDesc::Summary(SpeciesCount { species, count }) => {
                    let species = data.species_ref(species)?;
                    for _ in 0..*count {
                        counter += 1;
                        let growth = if options.assume_all_fish_fully_grown {
                            Growth::Final
                        } else {
                            species.earliest_growth_stage()
                        };
                        animals.push(AnimalRef {
                            id: counter,
                            species,
                            growth,
                        })
                    }
                }
                AnimalDesc::Individual(Animal { species, growth, .. }) => {
                    let species = data.species_ref(species)?;
                    counter += 1;
                    animals.push(AnimalRef {
                        id: counter,
                        species,
                        growth: *growth,
                    })
                }
            }
        }

        let min_tank = minimum_viable_tank(&animals);

        println!("{}:", exhibit.name);
        // TODO this isn't quite right if some fish are not grown
        // TODO TODO TODO
        //println!("- {}/{}, {}%", min_tank.size, exhibit.tank.size.0, min_tank.quality);

        for item in minimum_required_food(data, &animals) {
            println!("- {}x {}", item.count, item.food);
        }

        let exhibit_spec = ExhibitSpec {
            animals: &animals,
            environment: min_tank,
        };

        let violations = find_violations(&exhibit_spec);

        let mut messages: Vec<_> = violations.iter().map(|v| v.to_string()).collect();
        messages.sort();
        messages.dedup();

        for v in messages {
            was_problem = true;
            println!("- {}", v);
        }
    }

    if !was_problem {
        println!("No problems!");
    }

    Ok(())
}

pub fn try_expand_tank(base: &ExhibitRef, expansion: &ExhibitSpec) {}

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

// Guess at the minimum viable tank for the given species.
// Still requires checking for constraint violations.
fn minimum_viable_tank(animals: &[AnimalRef<'_>]) -> Environment {
    if animals.len() == 0 {
        panic!("need to specify at least some animals");
    }

    let constrained_size = animals.iter().map(|a| a.species.minimum_needed_tank_size()).max().unwrap();
    let summed_size: u16 = animals.iter().map(|a| a.species.maximum_size()).sum();
    let light = animals
        .iter()
        .filter_map(|s| match s.species.needs.light {
            Some(Need::Loves(x)) => Some(x),
            Some(Need::Dislikes) => Some(0),
            None => None,
        })
        .max();

    Environment {
        size: std::cmp::max(constrained_size, summed_size),
        temperature: animals[0].species.habitat.temperature,
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
