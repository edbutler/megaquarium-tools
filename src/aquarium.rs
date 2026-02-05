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

pub fn animals_to_counts(animals: &[AnimalRef]) -> Vec<SpeciesCount> {
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

pub fn fixtures_to_counts(fixtures: &[FixtureRef]) -> Vec<FixtureCount> {
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
    use crate::rules::RuleOptions;
    use crate::tank::test::test_tank_model;

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
}
