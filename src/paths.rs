use directories::UserDirs;
use std::path::{Path, PathBuf};

static POSSIBLE_DATA_DIRECTORIES: &'static [&str] = &["C:/Program Files (x86)/Steam", "D:/steam/"];

const LOCAL_DATA_PATH: &str = "steamapps/common/Megaquarium/Megaquarium_Data/GameData/";

pub const TANKS_PATH: &str = "Data/tanks.data";
pub const FISHES_PATH: &str = "Data/animals.data";
pub const CORALS_PATH: &str = "Data/corals.data";

pub fn find_data_dir() -> PathBuf {
    for d in POSSIBLE_DATA_DIRECTORIES {
        let potential = Path::new(d).join(LOCAL_DATA_PATH);
        if potential.exists() {
            return potential;
        }
    }

    panic!("Cannot find data directory");
}

pub fn find_save_dir() -> PathBuf {
    let dirs = UserDirs::new().unwrap();
    let docs = dirs.document_dir().unwrap();
    docs.join("My Games/Megaquarium/Saves")
}
