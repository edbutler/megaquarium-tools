use crate::animal::*;
use crate::tank::*;
use crate::paths::*;

use std::fmt;
use std::error::{Error};
use std::path::{Path};
use std::io::BufReader;
use std::fs::File;
use serde_json::{Value, from_reader};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct GameData {
    species: Vec<Species>,
    tanks: Vec<Tank>,
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
    let objects = &json["Objects"].as_array().ok_or(BadJson)?;

    Ok(Vec::new())
}

fn read_json(directory: &Path, file: &str) -> Result<Value> {
    let file = File::open(directory.join(file))?;
    let reader = BufReader::new(file);
    let result = from_reader(reader)?;
    Ok(result)
}