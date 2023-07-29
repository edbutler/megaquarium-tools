use std::env;
use directories::UserDirs;
use std::path::{Path, PathBuf};

static POSSIBLE_DATA_DIRECTORIES: &'static [&str] = &[
    "C:/Program Files (x86)/Steam/steamapps/common/Megaquarium/Megaquarium_Data/GameData",
    "D:/steam/steamapps/common/Megaquarium/Megaquarium_Data/GameData",
    "~/Library/Application Support/Steam/steamapps/common/Megaquarium/Megaquarium.app/Contents/GameData"
];

pub const TANKS_PATH: &str = "Data/tanks.data";
pub const FISHES_PATH: &str = "Data/animals.data";
pub const CORALS_PATH: &str = "Data/corals.data";

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
