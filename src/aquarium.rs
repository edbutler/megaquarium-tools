use crate::animal::*;
use crate::data::GameData;
use crate::rules::RuleOptions;
use crate::tank::*;
use crate::util::Result;

#[derive(Debug)]
pub struct Aquarium {
    pub exhibits: Vec<Exhibit>,
}

#[derive(Debug)]
pub struct AquariumRef<'a> {
    pub exhibits: Vec<ExhibitRef<'a>>,
}

#[derive(Debug)]
pub struct Exhibit {
    pub name: String,
    pub tank: Tank,
    pub animals: Vec<Animal>,
}

#[derive(Debug)]
pub struct ExhibitRef<'a> {
    pub name: String,
    pub tank: TankRef<'a>,
    pub animals: Vec<AnimalRef<'a>>,
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
                }
            })
            .collect();

        AquariumDesc { exhibits }
    }
}

impl AquariumDesc {
    pub fn to_aquarium<'a>(&self, data: &'a GameData, options: RuleOptions) -> Result<AquariumRef<'a>> {
        let mut counter = 0;

        let exhibits: Result<Vec<_>> = self.exhibits.iter().map(|exhibit| {
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
                        animals.push(AnimalRef {
                            id: counter,
                            species,
                            growth: *growth,
                        })
                    }
                }
            }

            Ok(ExhibitRef { name: exhibit.name.clone(), animals, tank })
        }).collect();

        Ok(AquariumRef { exhibits: exhibits? })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::animal::test::*;

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
}
