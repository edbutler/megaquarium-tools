use crate::animal::*;
use crate::data;
use crate::util::*;

pub struct CheckArgs {
    pub species: Vec<(String, u64)>,
    pub debug: bool,
}

pub fn check_for_viable_tank(data: &data::GameData, args: CheckArgs) -> Result<()> {
    let mut listing = Vec::new();

    for (s, count) in args.species {
        let species = lookup(&data, &s)?;
        listing.push(AnimalSpec {
            species: species.id.clone(),
            count,
        });
    }

    Ok(())
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
