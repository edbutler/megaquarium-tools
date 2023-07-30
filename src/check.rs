use crate::animal::*;
use crate::data;
use crate::tank::*;
use crate::util::*;
use crate::rules::*;

pub struct CheckArgs {
    pub species: Vec<(String, u16)>,
    pub debug: bool,
}

pub fn check_for_viable_tank(data: &data::GameData, args: CheckArgs) -> Result<()> {
    let mut listing = Vec::new();

    for (s, count) in args.species {
        let species = lookup(&data, &s)?;
        listing.push(AnimalSpec {
            species,
            count,
        });
    }

    Ok(())
}

fn minimum_viable_tank(spec: &ExhibitSpec<'_>) -> TankStatus {
    if spec.animals.len() == 0 {
        panic!("need to specify at least some animals");
    }

    let animals = spec.animals;

    let constrained_size = animals.iter().map(|s| s.species.minimum_needed_size()).max().unwrap();
    let summed_size: u16 = animals.iter().map(|s| s.species.size.final_size).sum();

    TankStatus {
        size: std::cmp::max(constrained_size, summed_size),
        environment: Environment {
            temperature: Temperature::Warm,
            salinity: Salinity::Salty,
            quality: 0,
        },
        lighting: 0,
        rounded: false,
    }
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
