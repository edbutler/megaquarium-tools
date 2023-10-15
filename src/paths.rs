use directories::UserDirs;
use std::env;
use std::path::{Path, PathBuf};

static POSSIBLE_DATA_DIRECTORIES: &'static [&str] = &[
    "C:/Program Files (x86)/Steam/steamapps/common/Megaquarium/Megaquarium_Data/GameData",
    "D:/steam/steamapps/common/Megaquarium/Megaquarium_Data/GameData",
    "~/Library/Application Support/Steam/steamapps/common/Megaquarium/Megaquarium.app/Contents/GameData",
];

pub const TANK_PATHS: &[&str] = &["Data/tanks.data"];
pub const FISH_PATHS: &[&str] = &["Data/animals.data", "Data/corals.data"];
pub const FOOD_PATHS: &[&str] = &[
    "Data/fishFood.data",
    "DLC/Freshwater Frenzy/Data/ff fishFood.data",
    "DLC/Architect's Collection/Data/ac fishFood.data",
];

pub fn find_data_dir() -> PathBuf {
    let dirs = UserDirs::new().unwrap();
    let home = dirs.home_dir();

    for d in POSSIBLE_DATA_DIRECTORIES {
        let directory = d.replace("~", &home.to_str().unwrap());

        let potential = Path::new(&directory);
        if potential.exists() {
            return potential.to_path_buf();
        }
    }

    panic!("Cannot find data directory");
}

pub fn find_save_dir() -> PathBuf {
    let dirs = UserDirs::new().unwrap();

    match env::consts::OS {
        "windows" => {
            let docs = dirs.document_dir().unwrap();
            docs.join("My Games/Megaquarium/Saves")
        }
        "macos" => {
            let home = dirs.home_dir();
            home.join("Library/Application Support/Megaquarium/Saves")
        }
        _ => {
            panic!("unsupported OS")
        }
    }
}
