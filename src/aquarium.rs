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
pub struct AquariumSpec {
    pub exhibits: Vec<ExhibitSpec>,
}

#[derive(Debug)]
pub struct ExhibitSpec {
    pub tank: TankSpec,
    pub animals: Vec<AnimalSpec>,
}

impl Aquarium<'_> {
    pub fn to_spec(&self) -> AquariumSpec {
        let exhibits = self
            .exhibits
            .iter()
            .map(|e| {
                let mut animals: HashMap<&str, u64> = HashMap::new();

                for a in &e.animals {
                    let count = animals.entry(&a.species.id).or_insert(0);
                    *count += 1;
                }

                ExhibitSpec {
                    tank: TankSpec {
                        model: e.tank.model.id.clone(),
                        size: e.tank.size,
                    },
                    animals: animals
                        .into_iter()
                        .map(|(k, v)| AnimalSpec {
                            species: k.to_string(),
                            count: v,
                        })
                        .collect(),
                }
            })
            .collect();

        AquariumSpec { exhibits: exhibits }
    }
}
