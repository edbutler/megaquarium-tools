// pattern: Functional Core

use crate::animal::*;
use crate::data::GameData;
use crate::fixture::{Fixture, FixtureRef};
use crate::rules::RuleOptions;
use crate::tank::*;
use crate::util::Result;

#[derive(Debug)]
pub struct AquariumRef<'a> {
    pub exhibits: Vec<ExhibitRef<'a>>,
}

#[derive(Debug)]
pub struct ExhibitRef<'a> {
    pub name: String,
    pub tank: TankRef<'a>,
    pub animals: Vec<AnimalRef<'a>>,
    pub fixtures: Vec<FixtureRef<'a>>,
}

#[derive(Debug)]
pub struct AquariumDesc {
    pub exhibits: Vec<ExhibitDesc>,
}

#[derive(Debug)]
pub struct ExhibitDesc {
    pub name: String,
    pub tank: Tank,
    pub animals: Vec<AnimalDesc>,
    pub fixtures: Vec<FixtureDesc>,
}

#[derive(Debug)]
pub enum AnimalDesc {
    Individual(Animal),
    Summary(SpeciesCount),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesCount {
    pub species: String,
    pub count: u16,
}

#[derive(Debug)]
pub enum FixtureDesc {
    Individual(Fixture),
    Summary(FixtureCount),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixtureCount {
    pub model: String,
    pub count: u16,
}

impl ExhibitRef<'_> {
    pub fn loaded_environment(&self) -> LoadedEnvironment {
        let f = &self.fixtures;
        let light = f.iter().map(|f| f.model.light.unwrap_or(0)).sum();
        let plants = f.iter().map(|f| f.model.plants.unwrap_or(0) as u16).sum();
        let rocks = f.iter().map(|f| f.model.rocks.unwrap_or(0) as u16).sum();
        let caves = f.iter().map(|f| f.model.caves.unwrap_or(0) as u16).sum();
        let bogwood = f.iter().map(|f| f.model.bogwood.unwrap_or(0) as u16).sum();
        let flat_surfaces = f.iter().map(|f| f.model.flat_surfaces.unwrap_or(0) as u16).sum();
        let vertical_surfaces = f.iter().map(|f| f.model.vertical_surfaces.unwrap_or(0) as u16).sum();
        let fluffy_foliage = f.iter().map(|f| f.model.fluffy_foliage.unwrap_or(0) as u16).sum();

        let mut distinct_models: Vec<&str> = f.iter().map(|f| f.model.id.as_str()).collect();
        distinct_models.sort();
        distinct_models.dedup();

        LoadedEnvironment {
            size: self.tank.volume(),
            light,
            plants,
            rocks,
            caves,
            bogwood,
            flat_surfaces,
            vertical_surfaces,
            fluffy_foliage,
            interior: self.tank.model.interior,
            different_decorations: distinct_models.len() as u8,
        }
    }
}

fn animals_to_counts(animals: &[AnimalRef]) -> Vec<SpeciesCount> {
    let mut animals: Vec<_> = animals.iter().collect();
    animals.sort_by_key(|a| &a.species.id);

    let mut result = Vec::new();

    let acc = animals.into_iter().fold(None, |acc, a| match acc {
        None => Some(SpeciesCount {
            species: a.species.id.clone(),
            count: 1,
        }),
        Some(mut c) => {
            if c.species == a.species.id {
                c.count += 1;
                Some(c)
            } else {
                result.push(c);
                Some(SpeciesCount {
                    species: a.species.id.clone(),
                    count: 1,
                })
            }
        }
    });

    if let Some(s) = acc {
        result.push(s);
    }

    result
}

fn fixtures_to_counts(fixtures: &[FixtureRef]) -> Vec<FixtureCount> {
    let mut fixtures: Vec<_> = fixtures.iter().collect();
    fixtures.sort_by_key(|f| &f.model.id);

    let mut result = Vec::new();

    let acc = fixtures.into_iter().fold(None, |acc, f| match acc {
        None => Some(FixtureCount {
            model: f.model.id.clone(),
            count: 1,
        }),
        Some(mut c) => {
            if c.model == f.model.id {
                c.count += 1;
                Some(c)
            } else {
                result.push(c);
                Some(FixtureCount {
                    model: f.model.id.clone(),
                    count: 1,
                })
            }
        }
    });

    if let Some(s) = acc {
        result.push(s);
    }

    result
}

impl AquariumRef<'_> {
    pub fn description(&self, summarize: bool) -> AquariumDesc {
        let exhibits = self
            .exhibits
            .iter()
            .map(|e| {
                let animals: Vec<_> = if summarize {
                    animals_to_counts(&e.animals).into_iter().map(AnimalDesc::Summary).collect()
                } else {
                    e.animals.iter().map(|a| AnimalDesc::Individual(a.to_animal())).collect()
                };

                ExhibitDesc {
                    name: e.name.clone(),
                    tank: Tank {
                        id: e.tank.id,
                        model: e.tank.model.id.clone(),
                        size: e.tank.size,
                    },
                    animals,
                    fixtures: if summarize {
                        fixtures_to_counts(&e.fixtures).into_iter().map(FixtureDesc::Summary).collect()
                    } else {
                        e.fixtures
                            .iter()
                            .map(|f| {
                                FixtureDesc::Individual(Fixture {
                                    id: f.id,
                                    model: f.model.id.clone(),
                                })
                            })
                            .collect()
                    },
                }
            })
            .collect();

        AquariumDesc { exhibits }
    }
}

impl AquariumDesc {
    pub fn to_ref<'a>(&self, data: &'a GameData, options: &RuleOptions) -> Result<AquariumRef<'a>> {
        let mut counter = 0;
        let mut fixture_counter: u64 = 0;

        let exhibits: Result<Vec<_>> = self
            .exhibits
            .iter()
            .map(|exhibit| {
                let tank = exhibit.tank.to_ref(data)?;

                let mut animals = Vec::new();

                for desc in &exhibit.animals {
                    match desc {
                        AnimalDesc::Summary(SpeciesCount { species, count }) => {
                            let species = data.species_ref(species)?;
                            for _ in 0..*count {
                                counter += 1;
                                let growth = if options.assume_all_fish_fully_grown {
                                    Growth::Final
                                } else {
                                    species.earliest_growth_stage()
                                };
                                animals.push(AnimalRef {
                                    id: counter,
                                    species,
                                    growth,
                                })
                            }
                        }
                        AnimalDesc::Individual(Animal { species, growth, .. }) => {
                            let species = data.species_ref(species)?;
                            counter += 1;
                            let effective_growth = if options.assume_all_fish_fully_grown {
                                Growth::Final
                            } else {
                                *growth
                            };
                            animals.push(AnimalRef {
                                id: counter,
                                species,
                                growth: effective_growth,
                            })
                        }
                    }
                }

                let mut fixtures = Vec::new();

                for desc in &exhibit.fixtures {
                    match desc {
                        FixtureDesc::Summary(FixtureCount { model, count }) => {
                            let model = data.fixture_ref(model)?;
                            for _ in 0..*count {
                                fixture_counter += 1;
                                fixtures.push(FixtureRef {
                                    id: fixture_counter,
                                    model,
                                });
                            }
                        }
                        FixtureDesc::Individual(Fixture { model, .. }) => {
                            let model = data.fixture_ref(model)?;
                            fixture_counter += 1;
                            fixtures.push(FixtureRef {
                                id: fixture_counter,
                                model,
                            });
                        }
                    }
                }

                Ok(ExhibitRef {
                    name: exhibit.name.clone(),
                    animals,
                    tank,
                    fixtures,
                })
            })
            .collect();

        Ok(AquariumRef { exhibits: exhibits? })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::animal::test::*;
    use crate::data::GameData;
    use crate::fixture::FixtureModel;
    use crate::rules::RuleOptions;
    use crate::tank::test::test_tank_model;
    use crate::tank::{Interior, LoadedEnvironment, TankModel, TankRef};

    #[test]
    fn test_animals_to_spec() {
        let species1 = test_species("capybara");
        let species2 = test_species("pika");
        let species3 = test_species("viscacha");

        let input = vec![
            AnimalRef {
                id: 1,
                species: &species1,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species2,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species2,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species1,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species3,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species3,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species1,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species2,
                growth: Growth::Final,
            },
            AnimalRef {
                id: 1,
                species: &species2,
                growth: Growth::Final,
            },
        ];

        let expected = vec![
            SpeciesCount {
                species: species1.id.clone(),
                count: 3,
            },
            SpeciesCount {
                species: species2.id.clone(),
                count: 4,
            },
            SpeciesCount {
                species: species3.id.clone(),
                count: 2,
            },
        ];

        let result = animals_to_counts(&input);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_ref_preserves_growth_when_flag_false() {
        let species = test_species_with_stages("goldfish");
        let tank_model = test_tank_model("basic_tank");
        let data = GameData {
            species: vec![species],
            tanks: vec![tank_model],
            fixtures: vec![],
            food: vec![],
        };

        let aquarium_desc = AquariumDesc {
            exhibits: vec![ExhibitDesc {
                name: "Tank1".to_string(),
                tank: Tank {
                    id: 1,
                    model: "basic_tank".to_string(),
                    size: (5, 5),
                },
                animals: vec![AnimalDesc::Individual(Animal {
                    id: 1,
                    species: "goldfish".to_string(),
                    growth: Growth::Growing { stage: 0, growth: 5 },
                })],
                fixtures: vec![],
            }],
        };

        let options = RuleOptions {
            assume_all_fish_fully_grown: false,
        };
        let result = aquarium_desc.to_ref(&data, &options).unwrap();

        assert_eq!(result.exhibits[0].animals[0].growth, Growth::Growing { stage: 0, growth: 5 });
    }

    #[test]
    fn test_to_ref_overrides_growth_when_flag_true() {
        let species = test_species_with_stages("goldfish");
        let tank_model = test_tank_model("basic_tank");
        let data = GameData {
            species: vec![species],
            tanks: vec![tank_model],
            fixtures: vec![],
            food: vec![],
        };

        let aquarium_desc = AquariumDesc {
            exhibits: vec![ExhibitDesc {
                name: "Tank1".to_string(),
                tank: Tank {
                    id: 1,
                    model: "basic_tank".to_string(),
                    size: (5, 5),
                },
                animals: vec![AnimalDesc::Individual(Animal {
                    id: 1,
                    species: "goldfish".to_string(),
                    growth: Growth::Growing { stage: 0, growth: 5 },
                })],
                fixtures: vec![],
            }],
        };

        let options = RuleOptions {
            assume_all_fish_fully_grown: true,
        };
        let result = aquarium_desc.to_ref(&data, &options).unwrap();

        assert_eq!(result.exhibits[0].animals[0].growth, Growth::Final);
    }

    #[test]
    fn test_loaded_environment() {
        let tank_model = TankModel {
            id: "round_tank".to_string(),
            min_size: (1, 1),
            max_size: (10, 10),
            double_density: 4,
            interior: Some(Interior::Rounded),
        };

        let fixture_model_a = FixtureModel {
            id: "coral_rock".to_string(),
            light: Some(3),
            plants: Some(2),
            rocks: Some(4),
            caves: Some(1),
            bogwood: Some(2),
            flat_surfaces: Some(3),
            vertical_surfaces: Some(1),
            fluffy_foliage: Some(2),
        };

        let fixture_model_b = FixtureModel {
            id: "tall_plant".to_string(),
            light: Some(1),
            plants: Some(5),
            rocks: Some(1),
            caves: Some(2),
            bogwood: Some(1),
            flat_surfaces: Some(1),
            vertical_surfaces: Some(3),
            fluffy_foliage: Some(4),
        };

        let exhibit = ExhibitRef {
            name: "Test Tank".to_string(),
            tank: TankRef {
                id: 1,
                model: &tank_model,
                size: (3, 5),
            },
            animals: vec![],
            fixtures: vec![
                FixtureRef { id: 1, model: &fixture_model_a },
                FixtureRef { id: 2, model: &fixture_model_b },
            ],
        };

        let result = exhibit.loaded_environment();

        assert_eq!(
            result,
            LoadedEnvironment {
                size: 30, // 3 * 5 * 4 / 2
                light: 4, // 3 + 1
                plants: 7, // 2 + 5
                rocks: 5, // 4 + 1
                caves: 3, // 1 + 2
                bogwood: 3, // 2 + 1
                flat_surfaces: 4, // 3 + 1
                vertical_surfaces: 4, // 1 + 3
                fluffy_foliage: 6, // 2 + 4
                interior: Some(Interior::Rounded),
                different_decorations: 2,
            }
        );
    }
}
