use crate::animal::*;
use crate::aquarium::*;
use crate::paths::*;
use crate::tank::*;

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct GameData {
    pub species: Vec<Species>,
    pub tanks: Vec<Tank>,
}

impl GameData {
    pub fn species_ref(&self, id: &str) -> Option<&Species> {
        for s in &self.species {
            if s.id.eq(id) {
                return Some(s);
            }
        }

        None
    }
}

pub fn read_game_data() -> Result<GameData> {
    let directory = find_data_dir();

    let result = GameData {
        species: read_species(&directory)?,
        tanks: Vec::new(),
    };

    Ok(result)
}

pub fn read_save<'a>(data: &'a GameData, save_name: &str) -> Result<Aquarium<'a>> {
    let directory = find_save_dir();
    let json = read_json(&directory, &(save_name.to_string() + ".sav"))?;

    let objects = json["objects"].as_array().ok_or("no objects")?;

    let mut animals: HashMap<u64, Vec<Animal<'a>>> = HashMap::new();
    let mut tanks: Vec<Tank> = Vec::new();

    for o in objects {
        let obj = o.as_object().ok_or("object is not json object")?;

        let is_in_game_world = match obj.get("inGameWorld") {
            Some(Value::Bool(true)) => true,
            _ => false,
        };

        if !is_in_game_world {
            continue;
        }

        if let Some(a) = obj.get("animal") {
            let id = o["uid"].as_u64().ok_or("no id")?;
            let species_id = o["specId"].as_str().ok_or("no specId")?;
            let species = data
                .species_ref(species_id)
                .ok_or(bad_json(format!("Unknown species {}", species_id)))?;

            let animal = Animal {
                id: id,
                species: species,
                age: uint_or_default(&a["growth"], 0)?.try_into()?, // TODO check this
            };
            let tank = o["hosting"]["host"]
                .as_u64()
                .ok_or(bad_json(format!("no host string for {}", id)))?;

            let vec = animals.entry(tank).or_insert(Vec::new());
            vec.push(animal);
        }

        if obj.contains_key("tank") {
            let blueprint_id = o["specId"].as_str().ok_or("no specId")?;
            let id = o["uid"].as_u64().ok_or("no specId")?;

            let tank = Tank {
                id: id,
            };

            tanks.push(tank);
        }
    }

    let exhibits = tanks
        .into_iter()
        .map(|t| {
            let a = match animals.remove(&t.id) {
                Some(list) => list,
                None => Vec::new(),
            };

            Exhibit { tank: t, animals: a }
        })
        .collect();

    Ok(Aquarium { exhibits: exhibits })
}

#[derive(Debug, Clone)]
struct BadJson {
    pub message: Option<String>,
}

const UBJ: BadJson = BadJson { message: None };

fn bad_json<S: Into<String>>(msg: S) -> BadJson {
    BadJson {
        message: Some(msg.into()),
    }
}

impl fmt::Display for BadJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match &self.message {
            Some(s) => &s,
            None => "Bad json for game data",
        };
        write!(f, "{}", msg)
    }
}

impl Error for BadJson {}

fn as_string_array(json: &Value) -> Result<Vec<&str>> {
    let jarr = json.as_array().ok_or(UBJ)?;
    let sarr: Result<Vec<&str>> = jarr
        .iter()
        .map(|t| {
            let s = t.as_str().ok_or(UBJ)?;
            Ok(s)
        })
        .collect();
    sarr
}

fn uint_or_default<T: TryFrom<u64>>(json: &Value, default: T) -> Result<T>
where
    <T as TryFrom<u64>>::Error: 'static + std::error::Error,
{
    match json {
        Value::Null => Ok(default),
        _ => Ok(json.as_u64().ok_or(bad_json("expected number"))?.try_into()?),
    }
}

fn uint_or_none<T: TryFrom<u64>>(json: &Value) -> Result<Option<T>>
where
    <T as TryFrom<u64>>::Error: 'static + std::error::Error,
{
    match json {
        Value::Null => Ok(None),
        _ => Ok(Some(json.as_u64().ok_or(bad_json("expected number"))?.try_into()?)),
    }
}

fn read_species(directory: &Path) -> Result<Vec<Species>> {
    let mut animals = read_species_file(&read_json(directory, FISHES_PATH)?)?;
    let mut corals = read_species_file(&read_json(directory, CORALS_PATH)?)?;
    animals.append(&mut corals);
    Ok(animals)
}

fn read_species_file(json: &Value) -> Result<Vec<Species>> {
    let objects = json["objects"].as_array().ok_or("no species objects")?;
    let species: Result<Vec<Species>> = objects.iter().map(|o| read_single_species(o)).collect();
    Ok(species?)
}

fn read_single_species(o: &Value) -> Result<Species> {
    let id = o["id"].as_str().ok_or("no id")?;
    let tags = as_string_array(&o["tags"])?;
    let animal = o["animal"].as_object().ok_or("no animal")?;
    let stats = animal["stats"].as_object().ok_or("no stats")?;

    fn has_stat(stats: &Map<String, Value>, stat: &str) -> bool {
        stats.contains_key(stat)
    }

    fn stat_number(stats: &Map<String, Value>, stat: &str, key: &str) -> Result<Option<u8>> {
        match stats.get(stat) {
            None => Ok(None),
            Some(v) => Ok(Some(v[key].as_u64().ok_or(UBJ)?.try_into()?)),
        }
    };

    fn one_of<T: Copy>(stats: &Map<String, Value>, potential: &[(&str, T)]) -> Result<Option<T>> {
        let mut result = None;

        for (stat, prop) in potential {
            if has_stat(stats, stat) {
                if result.is_some() {
                    return Err(Box::new(bad_json("Species has mutually exclusive properties")));
                }

                result = Some(*prop);
            }
        }

        Ok(result)
    }

    let size = {
        let raw_stages = animal["stages"].as_array().ok_or(UBJ)?;
        let stages = raw_stages
            .iter()
            .map(|s| {
                let size = uint_or_default(&s["size"], 0)?;
                let time = uint_or_none(&s["growthTime"])?;
                Ok((size, time))
            })
            .collect::<Result<Vec<(u8, Option<u8>)>>>()?;

        let mut last_duration = 0;

        Size {
            final_size: stages.last().ok_or(UBJ)?.0,
            armored: has_stat(stats, "armored"),
            stages: stages
                .iter()
                .take(stages.len() - 1)
                .map(|(sz, d)| {
                    let duration = d.ok_or(UBJ)?;
                    let result = Stage {
                        size: *sz,
                        duration: duration - last_duration,
                    };
                    last_duration = duration;
                    Ok(result)
                })
                .collect::<Result<Vec<Stage>>>()?,
        }
    };

    let environment = {
        let temperature = if has_stat(stats, "isTropical") {
            Ok(Temperature::Warm)
        } else if has_stat(stats, "isColdwater") {
            Ok(Temperature::Cold)
        } else {
            Err(bad_json("Unknown temperature"))
        };

        let salinity = Salinity::Fresh;

        let quality = stat_number(stats, "waterQuality", "value")?.ok_or(bad_json("no water quality"));

        Environment {
            temperature: temperature?,
            salinity: salinity,
            quality: quality?,
        }
    };

    let diet = {
        if let Some(e) = &stats.get("eats") {
            let food = e["item"].as_str().ok_or(UBJ)?.to_string();
            let period = uint_or_default(&e["daysBetweenFeed"], 0)? + 1;

            Diet::Food {
                food: food,
                period: period,
            }
        } else if stats.contains_key("scavenger") {
            Diet::Scavenger
        } else {
            Diet::DoesNotEat
        }
    };

    Ok(Species {
        id: id.to_string(),
        kind: tags[1].to_string(),
        size: size,
        environment: environment,
        diet: diet,
        shoaling: stat_number(stats, "shoaler", "req")?,
        fighting: one_of(stats, &[("wimp", Fighting::Wimp), ("bully", Fighting::Bully)])?,
        lighting: if has_stat(stats, "dislikesLights") {
            Some(Lighting::Disallows)
        } else if let Some(v) = stat_number(stats, "light", "value")? {
            Some(Lighting::Requires(v))
        } else {
            None
        },
        cohabitation: one_of(
            stats,
            &[
                ("dislikesConspecifics", Cohabitation::NoConspecifics),
                ("dislikesConsgeners", Cohabitation::NoCongeners),
                ("congenersOnly", Cohabitation::OnlyCongeners),
                ("dislikesFoodCompetitors", Cohabitation::NoFoodCompetitors),
            ],
        )?,
        tank: TankNeeds {
            active_swimmer: has_stat(stats, "activeSwimmer"),
            rounded_tank: has_stat(stats, "needsRounded"),
        },
        predation: Vec::new(),
    })
}

fn read_json(directory: &Path, file: &str) -> Result<Value> {
    // serde's parser is strict (the maintainers have "never seen json with comments" (lol)),
    // so we do some gross regex stuff to purge trailing commas and comments, since that's
    // easier than writing an entire parser or using some unmaintained library.
    lazy_static! {
        static ref RE1: Regex = Regex::new("//.*?\n").unwrap();
        static ref RE2: Regex = Regex::new(",([\r\n \t]*\\})").unwrap();
        static ref RE3: Regex = Regex::new(",([\r\n \t]*\\])").unwrap();
    }

    let file = fs::read_to_string(directory.join(file))?;
    let file = RE1.replace_all(&file, "\n");
    let file = RE2.replace_all(&file, "$1");
    let file = RE3.replace_all(&file, "$1");
    let result = from_str(&file)?;

    Ok(result)
}
