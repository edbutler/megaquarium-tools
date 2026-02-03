// pattern: Imperative Shell

use crate::animal::*;
use crate::aquarium::*;
use crate::fixture::*;
use crate::paths::*;
use crate::tank::*;
use crate::util::error;
use crate::util::Result;

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

pub struct GameData {
    pub species: Vec<Species>,
    pub tanks: Vec<TankModel>,
    pub fixtures: Vec<FixtureModel>,
    pub food: Vec<String>,
}

impl GameData {
    pub fn try_species_ref(&self, id: &str) -> Option<&Species> {
        for s in &self.species {
            if s.id.eq(id) {
                return Some(s);
            }
        }

        None
    }

    pub fn species_ref(&self, id: &str) -> Result<&Species> {
        self.try_species_ref(id).ok_or(error(format!("uknown species {}", id)))
    }

    pub fn species_search(&self, search_string: &str) -> Vec<&Species> {
        fn is_adult(s: &Species) -> bool {
            s.breeding != Breeding::NotFullyGrown
        }
        fuzzy_match_string(|s: &Species| &s.id, is_adult, search_string, self.species.as_slice())
    }

    pub fn try_tank_ref(&self, id: &str) -> Option<&TankModel> {
        for t in &self.tanks {
            if t.id.eq(id) {
                return Some(t);
            }
        }

        None
    }

    pub fn tank_ref(&self, id: &str) -> Result<&TankModel> {
        self.try_tank_ref(id).ok_or(error(format!("uknown tank {}", id)))
    }
}

fn fuzzy_match_string<'a, T, F: Fn(&T) -> &str, P: Fn(&T) -> bool>(f: F, predicate: P, search_string: &str, list: &'a [T]) -> Vec<&'a T> {
    let mut result = Vec::new();

    let parts: Vec<&str> = search_string.split(" ").collect();

    for x in list {
        let name = f(x);
        if predicate(x) && parts.iter().all(|p| name.contains(p)) {
            result.push(x);
        }
    }

    result
}

pub fn read_game_data() -> Result<GameData> {
    let directory = find_data_dir();

    let result = GameData {
        species: read_species(&directory)?,
        tanks: read_tank_models(&directory)?,
        fixtures: read_fixture_models(&directory)?,
        food: read_food(&directory)?,
    };

    Ok(result)
}

pub fn read_save<'a>(data: &'a GameData, save_name: &str) -> Result<AquariumRef<'a>> {
    let directory = find_save_dir();
    let json = read_json(&directory, &(save_name.to_string() + ".sav"))?;

    let objects = json["objects"].as_array().ok_or("no objects")?;

    let mut animals: HashMap<u64, Vec<AnimalRef<'a>>> = HashMap::new();
    let mut tanks: Vec<(String, TankRef)> = Vec::new();

    // sort the tank models by length of id so we always choose the longest prefix
    let mut models: Vec<&'a TankModel> = data.tanks.iter().map(|t| t).collect();
    models.sort_unstable_by_key(|t| -(t.id.len() as i32));

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
            let species = data.species_ref(species_id)?;

            let animal = AnimalRef {
                id: id,
                species: species,
                growth: read_growth(a, species)?,
            };
            let tank = o["hosting"]["host"]
                .as_u64()
                .ok_or(bad_json(format!("no host string for {}", id)))?;

            let vec = animals.entry(tank).or_insert(Vec::new());
            vec.push(animal);
        }

        if obj.contains_key("tank") {
            let id = o["uid"].as_u64().ok_or("no specId")?;
            // this string contains both the model and the size in one munged string
            // they look like "<tank-type-id>_<x-dim>-<y-dim>" (e.g., lagoon_tank_3_4)
            let spec_id = o["specId"].as_str().ok_or(bad_json("no specId"))?;

            let name = o["name"].as_str().ok_or(bad_json("no name"))?;

            let model = *models
                .iter()
                .find(|t| spec_id.starts_with(&t.id))
                .ok_or(bad_json("No tank model"))?;
            let size = {
                // strip off the prefix and then split on '_' to get the dimensions
                let string = &spec_id[model.id.len() + 1..];
                let parts: Vec<&str> = string.split('_').collect();
                if parts.len() == 2 {
                    let x: u16 = parts[0].parse()?;
                    let y: u16 = parts[1].parse()?;
                    (x, y)
                } else {
                    return Err(Box::new(bad_json("cannot extract dimensions")));
                }
            };

            let tank = TankRef {
                id: id,
                model: model,
                size: size,
            };

            tanks.push((name.to_string(), tank));
        }
    }

    let exhibits = tanks
        .into_iter()
        .map(|(name, tank)| {
            let animals = match animals.remove(&tank.id) {
                Some(list) => list,
                None => Vec::new(),
            };

            ExhibitRef { name, tank, animals }
        })
        .collect();

    Ok(AquariumRef { exhibits: exhibits })
}

fn read_growth(v: &Value, s: &Species) -> Result<Growth> {
    // growth is number of days along current stage, may be == state length if cannot growth due to tank size
    // so when converting to age, we need to cap it to `stage len - 1` or it will seem like it's the wrong stage
    // could consider adding flag to allow growth when possible

    let stage: u8 = uint_or_default(&v["stageNumber"], 0)?.try_into()?;
    let growth: u8 = uint_or_default(&v["growth"], 0)?.try_into()?;

    if (stage as usize) > s.size.stages.len() {
        return Err(Box::new(bad_json("stageNumber greater than number of stages!")));
    }

    if stage as usize == s.size.stages.len() {
        Ok(Growth::Final)
    } else {
        Ok(Growth::Growing { stage, growth })
    }
}

#[derive(Debug)]
struct WrappedError {
    pub message: String,
    pub inner: Box<dyn Error>,
}

impl fmt::Display for WrappedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.message, self.inner)
    }
}

impl Error for WrappedError {}

#[derive(Debug, Clone)]
struct BadJson {
    pub message: Option<String>,
}

const UBJ: BadJson = BadJson { message: None };

fn bad_json<S: Into<String>>(msg: S) -> BadJson {
    BadJson { message: Some(msg.into()) }
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

fn bool_or_default(json: &Value, default: bool) -> bool {
    match json {
        Value::Bool(b) => *b,
        _ => default,
    }
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
    let mut animals = Vec::new();

    for path in FISH_PATHS {
        let list = read_species_file(&read_json(directory, path)?)?;
        animals.extend(list);
    }

    Ok(animals)
}

fn read_species_file(json: &Value) -> Result<Vec<Species>> {
    let objects = json["objects"].as_array().ok_or("no species objects")?;
    let mut result = Vec::with_capacity(objects.len());
    for o in objects {
        match read_single_species_wrapper(o)? {
            Some(s) => result.push(s),
            None => (),
        };
    }
    Ok(result)
}

fn read_single_species_wrapper(o: &Value) -> Result<Option<Species>> {
    match read_single_species(o) {
        Ok(r) => Ok(r),
        Err(e) => {
            let id = o["id"].as_str().unwrap_or("unknown");
            let wrapped = WrappedError {
                message: format!("error reading species {}", id),
                inner: e,
            };
            Err(Box::new(wrapped))
        }
    }
}

fn stat_number(stats: &Map<String, Value>, stat: &str, key: &str) -> Result<Option<u8>> {
    match stats.get(stat) {
        None => Ok(None),
        Some(v) => Ok(Some(v[key].as_u64().ok_or(UBJ)?.try_into()?)),
    }
}

fn stat_value(stats: &Map<String, Value>, stat: &str) -> Result<Option<u8>> {
    stat_number(stats, stat, "value")
}

fn maybe_stat_value(stats: Option<&Map<String, Value>>, stat: &str) -> Result<Option<u8>> {
    match stats {
        Some(s) => stat_value(s, stat),
        None => Ok(None),
    }
}

fn read_single_species(o: &Value) -> Result<Option<Species>> {
    let obj = o.as_object().unwrap();

    let id = obj["id"].as_str().ok_or("no id")?;
    let animal = obj["animal"].as_object().ok_or("no animal")?;
    let stats = animal["stats"].as_object().ok_or("no stats")?;
    let empty_map = Map::new();
    let salinity_obj = obj.get("salinity").and_then(|x| x.as_object()).unwrap_or(&empty_map);

    let is_adult = !(id.ends_with(".egg") || id.ends_with(".fry"));

    let genus = {
        let tags = as_string_array(&o["tags"])?;
        if tags.len() < 2 { "unknown" } else { tags[1] }.to_string()
    };

    fn has_stat(stats: &Map<String, Value>, stat: &str) -> bool {
        stats.contains_key(stat)
    }

    fn has_true_value(stats: &Map<String, Value>, stat: &str) -> bool {
        stats.get(stat).and_then(|x| x.as_bool()).unwrap_or(false)
    }

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
            .collect::<Result<Vec<(u16, Option<u16>)>>>()?;

        let mut last_duration = 0;

        Size {
            final_size: stages.last().ok_or(UBJ)?.0,
            armored: has_stat(stats, "armored"),
            immobile: o.as_object().unwrap().contains_key("immobile"),
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

    let habitat = {
        let temperature = if has_stat(stats, "isTropical") {
            Ok(Temperature::Warm)
        } else if has_stat(stats, "isColdwater") {
            Ok(Temperature::Cold)
        } else {
            Err(bad_json("Unknown temperature"))
        }?;

        let salinity = if has_true_value(salinity_obj, "canGoInFreshwater") {
            if has_true_value(salinity_obj, "canGoInSaltwater") {
                None
            } else {
                Some(Salinity::Fresh)
            }
        } else {
            Some(Salinity::Salty)
        };

        let minimum_quality = stat_value(stats, "waterQuality")?.unwrap_or(0);

        let active_swimmer = has_stat(stats, "activeSwimmer");

        let territorial = has_stat(stats, "territorial");

        let tank = if has_stat(stats, "needsRounded") {
            Some(Interior::Rounded)
        } else if has_stat(stats, "needsKreisel") {
            Some(Interior::Kreisel)
        } else {
            None
        };

        Habitat {
            temperature,
            salinity,
            minimum_quality,
            active_swimmer,
            territorial,
            interior: tank,
        }
    };

    let diet = {
        if let Some(e) = &stats.get("eats") {
            let food = e["item"].as_str().ok_or(UBJ)?.to_string();
            let period = uint_or_default(&e["daysBetweenFeed"], 0)? + 1;

            Diet::Food { food, period }
        } else if stats.contains_key("scavenger") {
            Diet::Scavenger
        } else {
            Diet::DoesNotEat
        }
    };

    let needs = {
        let plants = stat_value(stats, "likesPlants")?.map(|x| Need::Loves(x));
        let rocks = stat_value(stats, "likesRocks")?.map(|x| Need::Loves(x));
        let caves = stat_value(stats, "likesCave")?;
        let bogwood = stat_value(stats, "likesBogwood")?;
        let flat_surfaces = stat_value(stats, "likesFlatSurface")?;
        let vertical_surfaces = stat_value(stats, "likesVerticalSurface")?;
        let fluffy_foliage = stat_value(stats, "likesFluffyFoliage")?;
        let open_space = stat_value(stats, "openSpace")?;
        let explorer = stat_value(stats, "explorer")?;

        let light = if has_stat(stats, "dislikesLights") {
            Some(Need::Dislikes)
        } else if let Some(v) = stat_value(stats, "light")? {
            Some(Need::Loves(v))
        } else {
            None
        };

        Needs {
            light,
            plants,
            rocks,
            caves,
            bogwood,
            flat_surfaces,
            vertical_surfaces,
            fluffy_foliage,
            open_space,
            explorer,
        }
    };

    let predation: Vec<PreyType> = if let Some(Value::Object(eater)) = &stats.get("eater") {
        let result: Result<Vec<PreyType>> = eater
            .keys()
            .map(|k| {
                let str = k.strip_suffix("Eater").ok_or(bad_json("no Eater suffix"))?;
                let typ = PreyType::from_str(str).ok_or(bad_json(format!("unknown prey type {}", str)))?;
                Ok(typ)
            })
            .collect();
        result?
    } else {
        Vec::new()
    };

    let prey_type = {
        if has_stat(stats, "baby") {
            Ok(PreyType::Baby)
        } else if has_stat(stats, "isFish") {
            Ok(PreyType::Fish)
        } else if has_stat(stats, "isStarfish") {
            Ok(PreyType::Starfish)
        } else if has_stat(stats, "isCrustacean") {
            Ok(PreyType::Crustacean)
        } else if has_stat(stats, "isStonyCoral") {
            Ok(PreyType::StonyCoral)
        } else if has_stat(stats, "isSoftCoral") {
            Ok(PreyType::SoftCoral)
        } else if has_stat(stats, "isClam") {
            Ok(PreyType::Clam)
        } else if has_stat(stats, "isGorgonian") {
            Ok(PreyType::Gorgonian)
        } else if has_stat(stats, "isAnemone") {
            Ok(PreyType::Anemone)
        } else {
            Err(bad_json("unknown prey type"))
        }
    }?;

    let shoaling = {
        match stat_number(stats, "shoaler", "req")? {
            Some(count) => {
                let x = stats["shoaler"].as_object().unwrap();
                Some(Shoaling {
                    count,
                    one_ok: has_true_value(x, "or1"),
                    two_ok: has_true_value(x, "or2"),
                })
            }
            None => None,
        }
    };

    let cohabitation = one_of(
        stats,
        &[
            ("dislikesConspecifics", Cohabitation::NoConspecifics),
            ("dislikesCongeners", Cohabitation::NoCongeners),
            ("congenersOnly", Cohabitation::OnlyCongeners),
            ("dislikesFoodCompetitors", Cohabitation::NoFoodCompetitors),
            ("pairsOnly", Cohabitation::PairsOnly),
        ],
    )?;

    let breeding = if is_adult {
        if has_stat(stats, "breeder") {
            let breeder = stats["breeder"].as_object().ok_or(bad_json("breeder is not object"))?;
            let baby = breeder
                .get("babySpec")
                .and_then(|b| b.as_str())
                .ok_or(bad_json("no babySpec"))?
                .to_string();
            Breeding::Breedable(Breedable { baby })
        } else {
            Breeding::CannotBread
        }
    } else {
        Breeding::NotFullyGrown
    };

    Ok(Some(Species {
        id: id.to_string(),
        genus,
        prey_type,
        size,
        habitat,
        diet,
        needs,
        greedy: has_stat(stats, "greedy"),
        shoaling,
        fighting: one_of(stats, &[("wimp", Fighting::Wimp), ("bully", Fighting::Bully)])?,
        nibbling: one_of(stats, &[("nibbleable", Nibbling::Nibbleable), ("nibbler", Nibbling::Nibbler)])?,
        cohabitation,
        predation,
        communal: stat_value(stats, "communal")?,
        breeding,
    }))
}

fn read_fixture_models(directory: &Path) -> Result<Vec<FixtureModel>> {
    let mut fixtures = Vec::new();

    for path in FIXTURE_PATHS {
        let json = read_json(directory, path)?;
        let objects = json["objects"].as_array().ok_or("no tank objects")?;
        for x in objects {
            match read_single_fixture_model(x)? {
                Some(fixture) => fixtures.push(fixture),
                None => (),
            };
        }
    }

    Ok(fixtures)
}

fn read_single_fixture_model(o: &Value) -> Result<Option<FixtureModel>> {
    let obj = o.as_object().unwrap();
    let id = obj["id"].as_str().ok_or("no id")?;

    // Filter to only include objects with "scenery" or "light" tags
    let tags = as_string_array(&o["tags"])?;
    if !tags.iter().any(|t| *t == "scenery" || *t == "light") {
        return Ok(None);
    }

    let stats = obj.get("aquascaping")
        .and_then(|a| a.as_object())
        .and_then(|a| a.get("stats"))
        .and_then(|s| s.as_object());

    let plants = maybe_stat_value(stats, "isPlant")?;
    let rocks = maybe_stat_value(stats, "isRock")?;
    let caves = maybe_stat_value(stats, "isCave")?;
    let bogwood = maybe_stat_value(stats, "isBogwood")?;
    let flat_surfaces = maybe_stat_value(stats, "isFlatSurface")?;
    let vertical_surfaces = maybe_stat_value(stats, "isVerticalSurface")?;
    let fluffy_foliage = maybe_stat_value(stats, "isFluffyFoliage")?;

    Ok(Some(FixtureModel {
        id: id.to_string(),
        light: None,
        plants,
        rocks,
        caves,
        bogwood,
        flat_surfaces,
        vertical_surfaces,
        fluffy_foliage,
    }))
}

fn read_tank_models(directory: &Path) -> Result<Vec<TankModel>> {
    let mut tanks = Vec::new();

    for path in TANK_PATHS {
        let json = read_json(directory, path)?;
        let objects = json["objects"].as_array().ok_or("no tank objects")?;
        for x in objects {
            let tank = read_single_tank_model(x)?;
            tanks.push(tank);
        }
    }

    Ok(tanks)
}

fn read_single_tank_model(o: &Value) -> Result<TankModel> {
    let id = o["id"].as_str().ok_or("no id")?;
    let tank = &o["tank"];

    fn as_u16(value: &Value) -> Result<u16> {
        Ok(value.as_u64().ok_or(bad_json("expected number"))?.try_into()?)
    }

    let read_size = |key: &str| -> Result<(u16, u16)> {
        let json = &o["multisize"][key];
        let w = as_u16(&json["m"])?;
        let h = as_u16(&json["n"])?;
        Ok((w, h))
    };

    let density = tank["volumePerTile"].as_f64().ok_or(UBJ)?;

    let interior = if bool_or_default(&tank["isRounded"], false) {
        Some(Interior::Rounded)
    } else if bool_or_default(&tank["isKreisel"], false) {
        Some(Interior::Kreisel)
    } else {
        None
    };

    Ok(TankModel {
        id: id.to_string(),
        min_size: read_size("minSize")?,
        max_size: read_size("baseSize")?,
        double_density: (2.0 * density).round() as u16,
        interior,
    })
}

fn read_food(directory: &Path) -> Result<Vec<String>> {
    let mut food = Vec::new();

    for path in FOOD_PATHS {
        let json = read_json(directory, path)?;
        let objects = json["objects"].as_array().ok_or("no tank objects")?;
        for x in objects {
            let value = read_single_food(x)?;
            match value {
                Some(v) => food.push(v),
                None => (),
            };
        }
    }

    Ok(food)
}

fn read_single_food(o: &Value) -> Result<Option<String>> {
    let id = o["id"].as_str().ok_or("no id")?;
    let tags = as_string_array(&o["tags"])?;

    if tags.iter().any(|t| *t == "animalFood") {
        Ok(Some(id.to_string()))
    } else {
        Ok(None)
    }
}

fn read_json(directory: &Path, file: &str) -> Result<Value> {
    // serde's parser is strict (the maintainers have "never seen json with comments" (lol)),
    // so we do some gross regex stuff to purge trailing commas and comments, since that's
    // easier than writing an entire parser or using some unmaintained library.
    lazy_static! {
        static ref REGEXES: [(Regex,&'static str); 5] = [
            // comments
            (Regex::new("//.*?\n").unwrap(), "\n"),
            (Regex::new("(?s)/\\*.*?\\*/").unwrap(), ""),
            // trailing commas (needs to be after comments)
            (Regex::new(",([\r\n \t]*\\})").unwrap(), "$1"),
            (Regex::new(",([\r\n \t]*\\])").unwrap(), "$1"),
            // multiline strings in tanks.data and scenery.data
            (Regex::new("(?s)\"map\":\\s*\".*?\"").unwrap(), "\"map\":\"\""),
        ];

    }

    let mut file = fs::read_to_string(directory.join(file))?;
    for (regex, replacement) in REGEXES.iter() {
        file = regex.replace_all(&file, *replacement).to_string();
    }

    let result = from_str(&file)?;

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::animal::test::*;

    fn test_data(species: Vec<Species>) -> GameData {
        GameData {
            species,
            tanks: vec![],
            fixtures: vec![],
            food: vec![],
        }
    }

    #[test]
    fn test_try_species_ref() {
        let data = test_data(vec![test_species("foo"), test_species("bar")]);
        let foo = &data.species[0];
        let bar = &data.species[1];

        assert_eq!(data.try_species_ref("foo"), Some(foo));
        assert_eq!(data.try_species_ref("fo"), None);
        assert_eq!(data.try_species_ref("bar"), Some(bar));
        assert_eq!(data.try_species_ref("baz"), None);
    }

    #[test]
    fn test_species_search() {
        let data = test_data(vec![
            test_species("2_crescent_earthen"),
            test_species("4_pancake_scuppernong"),
            test_species("7_violet_crescent"),
        ]);
        let two = &data.species[0];
        let four = &data.species[1];
        let seven = &data.species[2];

        assert_eq!(data.species_search("foo"), Vec::<&Species>::new());
        assert_eq!(data.species_search("pancake"), vec![four]);
        assert_eq!(data.species_search("pan scupp"), vec![four]);
        assert_eq!(data.species_search("olet"), vec![seven]);
        assert_eq!(data.species_search("then"), vec![two]);
        assert_eq!(data.species_search("cresc then"), vec![two]);
        assert_eq!(data.species_search("cresc"), vec![two, seven]);
        assert_eq!(data.species_search("e"), vec![two, four, seven]);
    }

    #[test]
    fn test_read_game_data() {
        let data = read_game_data().unwrap();
        let species = data.species_ref("11_yellow_tang").unwrap();
        assert_eq!(species.size.final_size, 5);
    }
}
