use crate::animal::*;
use crate::aquarium::*;
use crate::data::{self, GameData};
use crate::rules::*;
use crate::sexpr_format::*;
use crate::tank::*;
use crate::util::*;

pub struct CheckArgs<'a> {
    pub species: &'a Vec<(String, u16)>,
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

pub struct ValidateArgs {
    pub aquarium: AquariumDesc,
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

#[derive(Debug, Clone)]
pub struct BadCheck {
    pub message: String,
}

pub fn bad_check<S: Into<String>>(msg: S) -> BadCheck {
    BadCheck { message: msg.into() }
}

impl std::fmt::Display for BadCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BadCheck {}

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
                AnimalDesc::Summary { species, count } => {
                    let species = data.species_ref(species).ok_or(bad_check("invalid species"))?;
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
                AnimalDesc::Individual { species, growth } => {
                    let species = data.species_ref(species).ok_or(bad_check("invalid species"))?;
                    counter += 1;
                    animals.push(AnimalRef {
                        id: counter,
                        species,
                        growth: *growth,
                    })
                }
            }
        }

        let animal_spec = animals_to_spec(&animals);

        let min_tank = minimum_viable_tank(&animal_spec);

        println!("{}:", exhibit.name);
        // TODO this isn't quite right if some fish are not grown
        println!("- {}/{}, {}%", min_tank.size, exhibit.tank.size, min_tank.quality);

        for item in minimum_required_food(data, &animal_spec) {
            println!("- {}x {}", item.count, item.food);
        }

        let exhibit_spec = ExhibitSpec {
            options: &options,
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

pub struct CheckResult {
    pub violations: Vec<Violation>,
    pub food: Vec<FoodAmount>,
    pub environment: Environment,
}

impl CheckResult {
    pub fn is_okay(&self) -> bool {
        self.violations.len() == 0
    }
}

pub fn check_for_viable_tank<'a>(data: &'a data::GameData, args: &CheckArgs) -> Result<CheckResult> {
    let mut animals = Vec::new();

    for (s, count) in args.species {
        let species = lookup(&data, &s)?;
        animals.push(SpeciesSpec { species, count: *count });
    }

    let environment = minimum_viable_tank(&animals);

    let options = RuleOptions {
        assume_all_fish_fully_grown: args.assume_all_fish_fully_grown,
    };

    let exhibit = ExhibitSpec {
        options: &options,
        animals: &animals_from_spec(&animals, args.assume_all_fish_fully_grown),
        environment,
    };

    let violations = find_violations(&exhibit);

    let food = minimum_required_food(data, &animals);

    Ok(CheckResult {
        violations,
        food,
        environment,
    })
}

pub fn print_check_result(args: &CheckArgs, result: &CheckResult) {
    println!("For contents:");
    for (spec, count) in args.species {
        println!("- {}x {}", count, spec);
    }

    if result.is_okay() {
        println!("\nThe minimum viable tank is:");
        if args.debug {
            println!("{:#?}", result.environment);
        } else {
            println!(
                "{}",
                PrettyPrinted {
                    expr: result.environment.to_sexp()
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

pub fn try_expand_tank() {

}

fn animals_from_spec<'a>(animals: &[SpeciesSpec<'a>], assume_fully_grown: bool) -> Vec<AnimalRef<'a>> {
    let mut counter = 0;

    animals
        .iter()
        .flat_map(|s| {
            (0..s.count).map(move |_| {
                counter += 1;
                let growth = if assume_fully_grown {
                    Growth::Final
                } else {
                    s.species.earliest_growth_stage()
                };
                AnimalRef {
                    id: counter,
                    species: s.species,
                    growth,
                }
            })
        })
        .collect()
}

// Guess at the minimum viable tank for the given species.
// Still requires checking for constraint violations.
fn minimum_viable_tank(species: &[SpeciesSpec<'_>]) -> Environment {
    if species.len() == 0 {
        panic!("need to specify at least some animals");
    }

    let constrained_size = species.iter().map(|s| s.species.minimum_needed_tank_size()).max().unwrap();
    let summed_size: u16 = species.iter().map(|s| s.count * s.species.maximum_size()).sum();
    let light = species
        .iter()
        .filter_map(|s| match s.species.needs.light {
            Some(Need::Loves(x)) => Some(x),
            Some(Need::Dislikes) => Some(0),
            None => None,
        })
        .max();

    Environment {
        size: std::cmp::max(constrained_size, summed_size),
        temperature: species[0].species.habitat.temperature,
        quality: species.iter().map(|s| s.species.habitat.minimum_quality).max().unwrap(),
        plants: minimum_need(species, |s| s.needs.plants),
        rocks: minimum_need(species, |s| s.needs.rocks),
        caves: minimum_need(species, |s| s.needs.caves.map(|x| Need::Loves(x))),
        light,
        interior: species.iter().find_map(|s| s.species.habitat.interior),
    }
}

pub struct FoodAmount {
    pub food: String,
    pub count: u16,
}

fn minimum_required_food(data: &GameData, species: &[SpeciesSpec<'_>]) -> Vec<FoodAmount> {
    let diets: Vec<(&String, u16)> = species
        .iter()
        .filter_map(|s| match &s.species.diet {
            Diet::Food { food, period: _ } => Some((food, s.count * s.species.amount_food_eaten())),
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

fn minimum_need<F: Fn(&Species) -> Option<Need>>(list: &[SpeciesSpec], f: F) -> Option<u16> {
    let foldfn = |acc: Option<u16>, s: &SpeciesSpec| -> Option<u16> {
        match f(s.species) {
            Some(Need::Dislikes) => Some(0),
            Some(Need::Loves(x)) => Some((x as u16) * s.count + acc.unwrap_or(0)),
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
