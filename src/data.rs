use crate::animal::*;
use crate::paths::*;
use crate::tank::*;

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, Value};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct GameData {
    pub species: Vec<Species>,
    pub tanks: Vec<Tank>,
}

pub fn read_game_data() -> Result<GameData> {
    let directory = find_data_dir();

    println!("Directory: {:?}", directory);

    let result = GameData {
        species: read_species(&directory)?,
        tanks: Vec::new(),
    };

    Ok(result)
}

#[derive(Debug, Clone)]
struct BadJson {
    pub message: Option<&'static str>,
}

const UBJ: BadJson = BadJson { message: None };

fn bad_json(msg: &'static str) -> BadJson {
    BadJson { message: Some(msg) }
}

impl fmt::Display for BadJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = self.message.unwrap_or("Bad json for game data");
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
    let json = read_json(directory, ANIMAL_PATH)?;
    let objects = json["objects"].as_array().ok_or("no species objects")?;

    let species: Result<Vec<Species>> = objects
        .iter()
        .map(|o| {
            let id = o["id"].as_str().ok_or("no id")?;
            let tags = as_string_array(&o["tags"])?;
            let animal = o["animal"].as_object().ok_or("no animal")?;
            let stats = animal["stats"].as_object().ok_or("no stats")?;

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

                Size {
                    final_size: stages.last().ok_or(UBJ)?.0,
                    stages: stages
                        .iter()
                        .take(stages.len() - 1)
                        .map(|(sz, d)| {
                            Ok(Stage {
                                size: *sz,
                                duration: d.ok_or(UBJ)?,
                            })
                        })
                        .collect::<Result<Vec<Stage>>>()?,
                }
            };

            let environment = {
                let temperature = if stats.contains_key("isTropical") {
                    Ok(Temperature::Warm)
                } else if stats.contains_key("isColdwater") {
                    Ok(Temperature::Cold)
                } else {
                    Err(bad_json("Unknown temperature"))
                };

                let salinity = Salinity::Fresh;

                let quality = stats["waterQuality"]["value"]
                    .as_u64()
                    .ok_or(bad_json("Unknown water quality"));

                Environment {
                    temperature: temperature?,
                    salinity: salinity,
                    quality: quality?.try_into()?,
                }
            };

            let diet = {
                match &stats.get("eats") {
                    Some(Value::Object(e)) => {
                        let food = e["item"].as_str().ok_or(UBJ)?.to_string();
                        let period = match e.get("daysBetweenFeed") {
                            Some(v) => v.as_u64().ok_or(UBJ)? + 1,
                            None => 1,
                        };

                        Diet::Food {
                            food: food,
                            period: period.try_into()?,
                        }
                    }
                    _ => {
                        if stats.contains_key("scavenger") {
                            Diet::Scavenger
                        } else {
                            Diet::DoesNotEat
                        }
                    }
                }
            };

            Ok(Species {
                id: id.to_string(),
                kind: tags[1].to_string(),
                size: size,
                environment: environment,
                diet: diet,
            })
        })
        .collect();

    Ok(species?)
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
