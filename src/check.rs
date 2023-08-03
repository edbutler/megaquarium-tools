use crate::animal::*;
use crate::data;
use crate::data::GameData;
use crate::tank::*;
use crate::util::*;
use crate::rules::*;
use crate::sexpr_format::*;

pub struct CheckArgs {
    pub species: Vec<(String, u16)>,
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

pub fn check_for_viable_tank(data: &data::GameData, args: CheckArgs) -> Result<()> {
    let mut animals = Vec::new();

    for (s, count) in args.species {
        let species = lookup(&data, &s)?;
        animals.push(SpeciesSpec {
            species,
            count,
        });
    }

    let tank = minimum_viable_tank(&animals);

    let options = RuleOptions {
        assume_all_fish_fully_grown: args.assume_all_fish_fully_grown,
    };

    let exhibit = ExhibitSpec {
        options,
        animals: &animals,
        tank
    };

    let violations = find_violations(&exhibit);

    println!("For contents:");
    for a in &animals {
        println!("- {}x {}", a.count, a.species.id);
    }

    if violations.len() > 0 {
        println!("\nA valid tank is not possible:");
        for v in violations {
            println!("- {}", v);
        }
    } else {
        println!("\nThe minimum viable tank is:");
        if args.debug {
            println!("{:#?}", exhibit.tank);
        } else {
            println!("{}", PrettyPrinted { expr: exhibit.tank.to_sexp() });
        }

        println!("\nWill require food (average per day):");
        for item in minimum_required_food(data, &animals) {
            println!("- {}x {}", item.count, item.food);
        }
    }


    Ok(())
}

// Guess at the minimum viable tank for the given species.
// Still requires checking for constraint violations.
fn minimum_viable_tank(species: &[SpeciesSpec<'_>]) -> Environment {
    if species.len() == 0 {
        panic!("need to specify at least some animals");
    }

    let constrained_size = species.iter().map(|s| s.species.minimum_needed_tank_size()).max().unwrap();
    let summed_size: u16 = species.iter().map(|s| s.count * s.species.maximum_size()).sum();
    let light = species.iter().filter_map(|s| {
        match s.species.needs.light {
            Some(Need::Loves(x)) => Some(x),
            Some(Need::Dislikes) => Some(0),
            None => None,
        }
    }).max();

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

struct FoodAmount {
    pub food: String,
    pub count: u16,
}

fn minimum_required_food(data: &GameData, species: &[SpeciesSpec<'_>]) -> Vec<FoodAmount> {
    let diets: Vec<(&String, u16)> = species.iter().filter_map(|s| {
        match &s.species.diet {
            Diet::Food { food, period: _ } => {
                Some((food, s.count * s.species.amount_food_eaten()))
            }
            _ => None
        }
    }).collect();

    data.food.iter().filter_map(|food| {
        let count = diets.iter().filter_map(|(x, c)| if food == *x { Some(c) } else { None }).sum();
        if count > 0 {
            Some(FoodAmount { food: food.clone(), count })
        } else {
            None
        }
    }).collect()
}

fn minimum_need<F: Fn(&Species)->Option<Need>>(list: &[SpeciesSpec], f: F) -> Option<u16> {
    let foldfn = |acc: Option<u16>, s:&SpeciesSpec| -> Option<u16> {
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
