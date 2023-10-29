use crate::animal::*;
use crate::tank::*;

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
    pub tank: TankDesc,
    pub animals: Vec<AnimalDesc>,
}

#[derive(Debug)]
pub enum AnimalDesc {
    Individual { species: String, growth: Growth },
    Summary { species: String, count: u16 },
}

#[derive(Debug)]
pub struct TankDesc {
    pub model: String,
    pub size: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesSpec<'a> {
    pub species: &'a Species,
    pub count: u16,
}

pub fn animals_to_spec<'a>(animals: &[AnimalRef<'a>]) -> Vec<SpeciesSpec<'a>> {
    let mut animals: Vec<_> = animals.iter().collect();
    animals.sort_by_key(|a| &a.species.id);

    let mut result = Vec::new();

    let acc = animals.into_iter().fold(None, |acc, a| match acc {
        None => Some(SpeciesSpec {
            species: a.species,
            count: 1,
        }),
        Some(mut spec) => {
            if std::ptr::eq(spec.species, a.species) {
                spec.count += 1;
                Some(spec)
            } else {
                result.push(spec);
                Some(SpeciesSpec {
                    species: a.species,
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
                    animals_to_spec(&e.animals)
                        .iter()
                        .map(|spec| AnimalDesc::Summary {
                            species: spec.species.id.to_string(),
                            count: spec.count,
                        })
                        .collect()
                } else {
                    e.animals
                        .iter()
                        .map(|a| AnimalDesc::Individual {
                            species: a.species.id.to_string(),
                            growth: a.growth,
                        })
                        .collect()
                };

                ExhibitDesc {
                    name: e.name.clone(),
                    tank: TankDesc {
                        model: e.tank.model.id.clone(),
                        size: e.tank.volume(),
                    },
                    animals,
                }
            })
            .collect();

        AquariumDesc { exhibits }
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
            SpeciesSpec {
                species: &species1,
                count: 3,
            },
            SpeciesSpec {
                species: &species2,
                count: 4,
            },
            SpeciesSpec {
                species: &species3,
                count: 2,
            },
        ];

        let result = animals_to_spec(&input);

        assert_eq!(result, expected);
    }
}
