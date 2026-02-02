// pattern: Functional Core

use crate::animal::*;
use crate::aquarium::*;
use crate::data::{self, GameData};
use crate::rules::*;
use crate::tank::*;
use crate::util::*;

pub struct CheckArgs<'a> {
    pub species: &'a [SpeciesCount],
    pub debug: bool,
    pub assume_all_fish_fully_grown: bool,
}

pub struct CheckQuery<'a> {
    pub debug: bool,
    pub counts: Vec<SpeciesCount>,
    pub animals: Vec<AnimalRef<'a>>,
}

pub struct ExhibitCheckResult {
    pub violations: Vec<Violation>,
    pub food: Vec<FoodAmount>,
    pub minimum_viable_environment: Environment,
}

impl ExhibitCheckResult {
    pub fn is_okay(&self) -> bool {
        self.violations.is_empty()
    }
}

#[derive(Debug)]
pub struct ExhibitValidation {
    pub name: String,
    pub tank_volume: u16,
    pub minimum_viable_environment: Environment,
    pub food: Vec<FoodAmount>,
    pub violations: Vec<Violation>,
}

#[derive(Debug)]
pub struct AquariumCheckResult {
    pub exhibits: Vec<ExhibitValidation>,
}

impl AquariumCheckResult {
    pub fn is_okay(&self) -> bool {
        self.exhibits.iter().all(|e| e.violations.is_empty())
    }
}

pub struct ValidateArgs<'a> {
    pub aquarium: &'a AquariumRef<'a>,
}

pub fn validate_aquarium(data: &data::GameData, args: &ValidateArgs) -> AquariumCheckResult {
    let mut exhibits = Vec::new();

    for exhibit in &args.aquarium.exhibits {
        if exhibit.animals.is_empty() {
            continue;
        }

        let minimum_viable_environment = minimum_viable_tank(&exhibit.animals);
        let food = minimum_required_food(data, &exhibit.animals);

        let exhibit_spec = ExhibitSpec {
            animals: &exhibit.animals,
            environment: minimum_viable_environment,
        };

        let violations = find_violations(&exhibit_spec);

        exhibits.push(ExhibitValidation {
            name: exhibit.name.clone(),
            tank_volume: exhibit.tank.volume(),
            minimum_viable_environment,
            food,
            violations,
        });
    }

    AquariumCheckResult { exhibits }
}

pub fn check_for_viable_tank<'a>(data: &GameData, animals: &[AnimalRef]) -> ExhibitCheckResult {
    let environment = minimum_viable_tank(&animals);
    let exhibit = ExhibitSpec { animals, environment };
    let violations = find_violations(&exhibit);
    let food = minimum_required_food(data, &exhibit.animals);

    ExhibitCheckResult {
        violations,
        food,
        minimum_viable_environment: environment,
    }
}

pub fn try_expand_tank(data: &GameData, base: &ExhibitRef, expansion: &ExhibitSpec) -> ExhibitCheckResult {
    let mut animals = base.animals.clone();
    animals.extend(expansion.animals);
    check_for_viable_tank(data, &animals)
}

/// Builds a CheckQuery from user arguments by resolving species names.
pub fn create_check_query<'a>(data: &'a GameData, args: &CheckArgs) -> Result<CheckQuery<'a>> {
    let mut counter = 0;
    let capacity: u16 = args.species.iter().map(|c| c.count).sum();
    let mut counts = Vec::with_capacity(args.species.len());
    let mut animals = Vec::with_capacity(capacity as usize);

    for c in args.species {
        let species = lookup(data, &c.species)?;

        // copy resolved name into query for printing, since arg name is just a search string
        counts.push(SpeciesCount {
            species: species.id.clone(),
            count: c.count,
        });

        for _ in 0..c.count {
            counter += 1;
            let growth = if args.assume_all_fish_fully_grown {
                Growth::Final
            } else {
                species.earliest_growth_stage()
            };
            animals.push(AnimalRef {
                id: counter,
                species: species,
                growth,
            });
        }
    }

    Ok(CheckQuery {
        debug: args.debug,
        counts,
        animals,
    })
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
        light,
        plants: minimum_need(animals, |s| s.needs.plants),
        rocks: minimum_need(animals, |s| s.needs.rocks),
        caves: minimum_loves(animals, |s| s.needs.caves),
        bogwood: minimum_loves(animals, |s| s.needs.bogwood),
        flat_surfaces: minimum_loves(animals, |s| s.needs.flat_surfaces),
        vertical_surfaces: minimum_loves(animals, |s| s.needs.vertical_surfaces),
        fluffy_foliage: minimum_loves(animals, |s| s.needs.fluffy_foliage),
        open_space: animals.iter().filter_map(|s| s.species.needs.open_space).max(),
        interior: animals.iter().find_map(|s| s.species.habitat.interior),
        different_decorations: animals.iter().filter_map(|s| s.species.needs.explorer).max(),
    }
}

#[derive(Debug)]
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

fn minimum_loves<F: Fn(&Species) -> Option<u8>>(list: &[AnimalRef], f: F) -> Option<u16> {
    let foldfn = |acc: Option<u16>, a: &AnimalRef| -> Option<u16> {
        match f(a.species) {
            Some(x) => Some((x as u16) + acc.unwrap_or(0)),
            None => acc,
        }
    };

    list.iter().fold(None, foldfn)
}

/// Pure lookup: searches the in-memory species list by fuzzy match.
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::animal::test::test_species;
    use crate::animal::Growth;
    use crate::tank::test::test_tank_model;

    #[test]
    fn test_happy_path_single_tank_compatible_fish() {
        let species1 = test_species("clownfish");
        let species2 = test_species("damselfish");

        let tank_model = test_tank_model("basic_tank");

        let data = GameData {
            species: vec![species1.clone(), species2.clone()],
            tanks: vec![tank_model.clone()],
            food: vec![],
        };

        let animals = vec![
            AnimalRef {
                id: 1,
                species: &data.species[0],
                growth: Growth::Final,
            },
            AnimalRef {
                id: 2,
                species: &data.species[1],
                growth: Growth::Final,
            },
        ];

        let tank_ref = TankRef {
            id: 1,
            model: &data.tanks[0],
            size: (5, 5),
        };

        let exhibit = ExhibitRef {
            name: "Test Tank".to_string(),
            tank: tank_ref,
            animals,
        };

        let aquarium = AquariumRef { exhibits: vec![exhibit] };

        let args = ValidateArgs { aquarium: &aquarium };

        let result = validate_aquarium(&data, &args);
        assert!(result.is_okay());
        assert_eq!(result.exhibits.len(), 1);
        assert_eq!(result.exhibits[0].name, "Test Tank");
        assert!(result.exhibits[0].violations.is_empty());
    }

    #[test]
    fn test_aquarium_with_temperature_violation() {
        // Create species that require different temperatures
        // test_species creates warm fish by default
        let warm_species = test_species("warm_fish");
        let mut cold_species = test_species("cold_fish");
        cold_species.habitat.temperature = crate::tank::Temperature::Cold;

        let tank_model = test_tank_model("basic_tank");

        let data = GameData {
            species: vec![warm_species.clone(), cold_species.clone()],
            tanks: vec![tank_model.clone()],
            food: vec![],
        };

        let animals = vec![
            AnimalRef {
                id: 1,
                species: &data.species[0], // warm
                growth: Growth::Final,
            },
            AnimalRef {
                id: 2,
                species: &data.species[1], // cold
                growth: Growth::Final,
            },
        ];

        let tank_ref = TankRef {
            id: 1,
            model: &data.tanks[0],
            size: (5, 5),
        };

        let exhibit = ExhibitRef {
            name: "Mixed Temperature Tank".to_string(),
            tank: tank_ref,
            animals,
        };

        let aquarium = AquariumRef { exhibits: vec![exhibit] };

        let args = ValidateArgs { aquarium: &aquarium };

        let result = validate_aquarium(&data, &args);

        // The aquarium should have violations due to temperature incompatibility
        assert!(!result.is_okay());
        assert_eq!(result.exhibits.len(), 1);
        assert!(!result.exhibits[0].violations.is_empty());
    }

    #[test]
    fn test_validate_aquarium_captures_violation_details() {
        // Create species with incompatible salinity requirements
        // test_species creates salty fish by default
        let salty_species = test_species("salty_fish");
        let mut freshwater_species = test_species("freshwater_fish");
        freshwater_species.habitat.salinity = Some(crate::tank::Salinity::Fresh);

        let tank_model = test_tank_model("basic_tank");

        let data = GameData {
            species: vec![salty_species.clone(), freshwater_species.clone()],
            tanks: vec![tank_model.clone()],
            food: vec![],
        };

        let animals = vec![
            AnimalRef {
                id: 1,
                species: &data.species[0], // salty
                growth: Growth::Final,
            },
            AnimalRef {
                id: 2,
                species: &data.species[1], // freshwater
                growth: Growth::Final,
            },
        ];

        let tank_ref = TankRef {
            id: 1,
            model: &data.tanks[0],
            size: (5, 5),
        };

        let exhibit = ExhibitRef {
            name: "Mixed Salinity Tank".to_string(),
            tank: tank_ref,
            animals,
        };

        let aquarium = AquariumRef { exhibits: vec![exhibit] };

        let args = ValidateArgs { aquarium: &aquarium };

        let result = validate_aquarium(&data, &args);

        // Verify aquarium-level is_okay() reflects violations
        assert!(!result.is_okay());

        // Verify exhibit-level data
        let exhibit_result = &result.exhibits[0];
        assert_eq!(exhibit_result.name, "Mixed Salinity Tank");

        // Verify we have at least one violation captured
        assert!(
            !exhibit_result.violations.is_empty(),
            "Expected violations for incompatible salinity"
        );

        // Verify the minimum environment is calculated
        assert!(
            exhibit_result.minimum_viable_environment.size > 0,
            "Expected minimum viable environment to be calculated"
        );
    }
}
