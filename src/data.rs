use crate::animal::*;
use crate::tank::*;
use crate::paths::*;

use std::fmt;
use std::error::{Error};
use std::path::{Path};
use std::fs;
use serde_json::{Value, from_str};
use lazy_static::lazy_static;
use regex::Regex;

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
struct BadJson;

impl fmt::Display for BadJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad json format for game data")
    }
}

impl Error for BadJson {}

fn read_species(directory: &Path) -> Result<Vec<Species>> {
    let json = read_json(directory, ANIMAL_PATH)?;
    let objects = json["objects"].as_array().ok_or(BadJson)?;


    let species: Result<Vec<Species>> = objects.iter().map(|o| {
        let id = o["id"].as_str().ok_or(BadJson)?;

        Ok(Species {
            id: id.to_string(),
        })
    }).collect();

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