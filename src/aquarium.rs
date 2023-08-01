use crate::animal::*;
use crate::tank::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Aquarium<'a> {
    pub exhibits: Vec<Exhibit<'a>>,
}

#[derive(Debug)]
pub struct Exhibit<'a> {
    pub tank: Tank<'a>,
    pub animals: Vec<Animal<'a>>,
}

#[derive(Debug)]
pub struct AquariumDesc {
    pub exhibits: Vec<ExhibitDesc>,
}

#[derive(Debug)]
pub struct ExhibitDesc {
    pub tank: TankDesc,
    pub animals: Vec<AnimalDesc>,
}

#[derive(Debug)]
pub struct AnimalDesc {
    pub species: String,
    pub count: u16,
}

#[derive(Debug)]
pub struct TankDesc {
    pub model: String,
    pub size: u16,
}

impl Aquarium<'_> {
    pub fn description(&self) -> AquariumDesc {
        let exhibits = self
            .exhibits
            .iter()
            .map(|e| {
                let mut animals: HashMap<&str, u16> = HashMap::new();

                for a in &e.animals {
                    let count = animals.entry(&a.species.id).or_insert(0);
                    *count += 1;
                }

                ExhibitDesc {
                    tank: TankDesc {
                        model: e.tank.model.id.clone(),
                        size: e.tank.volume(),
                    },
                    animals: animals
                        .into_iter()
                        .map(|(k, v)| AnimalDesc {
                            species: k.to_string(),
                            count: v,
                        })
                        .collect(),
                }
            })
            .collect();

        AquariumDesc { exhibits: exhibits }
    }
}
