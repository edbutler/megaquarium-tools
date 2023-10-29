use crate::data::GameData;
use crate::rules::Constraint;
use crate::tank::{Interior, Temperature};
use crate::util::{as_str_display, Result};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Growth {
    Final,
    // if growth >= stage length, means that the animal should have grown but the tank size stopped it
    Growing { stage: u8, growth: u8 },
}

pub type AnimalId = u64;

#[derive(Debug, PartialEq, Clone)]
pub struct Animal {
    pub id: AnimalId,
    pub species: String,
    pub growth: Growth,
}

impl Animal {
    pub fn to_ref<'a>(&self, data: &'a GameData) -> Result<AnimalRef<'a>> {
        let species = data.species_ref(&self.species)?;
        Ok(AnimalRef {
            id: self.id,
            species,
            growth: self.growth,
        })
    }
}

#[derive(Debug)]
pub struct AnimalRef<'a> {
    pub id: AnimalId,
    pub species: &'a Species,
    pub growth: Growth,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Species {
    pub id: String,
    pub genus: String,
    pub prey_type: PreyType,
    pub size: Size,
    pub habitat: Habitat,
    pub diet: Diet,
    pub needs: Needs,
    pub greedy: bool,
    pub shoaling: Option<u8>,
    pub fighting: Option<Fighting>,
    pub cohabitation: Option<Cohabitation>,
    pub predation: Vec<PreyType>,
}

impl AnimalRef<'_> {
    pub fn to_animal(&self) -> Animal {
        Animal {
            id: self.id,
            species: self.species.id.clone(),
            growth: self.growth,
        }
    }

    pub fn size(&self) -> u16 {
        match self.growth {
            Growth::Final => self.species.size.final_size,
            Growth::Growing { stage, .. } => {
                assert!((stage as usize) < self.species.size.stages.len());
                self.species.size.stages[stage as usize].size
            }
        }
    }
}

impl Species {
    pub fn is_bully(&self) -> bool {
        self.fighting == Some(Fighting::Bully)
    }

    pub fn minimum_needed_tank_size(&self) -> u16 {
        if self.size.immobile {
            0
        } else {
            let size = self.size.final_size;
            if self.habitat.active_swimmer {
                6 * size
            } else {
                size
            }
        }
    }

    pub fn maximum_size(&self) -> u16 {
        if self.size.immobile {
            0
        } else {
            self.size.final_size
        }
    }

    pub fn earliest_growth_stage(&self) -> Growth {
        if self.size.stages.len() > 0 {
            Growth::Growing { stage: 0, growth: 0 }
        } else {
            Growth::Final
        }
    }

    pub fn needs_light(&self) -> bool {
        match self.needs.light {
            Some(Need::Loves(_)) => true,
            _ => false,
        }
    }

    pub fn predation_size(&self) -> u16 {
        let size = self.size.final_size;
        // number from https://steamcommunity.com/app/600480/discussions/0/3276824488724294545/
        (0.4 * (size as f64)).floor() as u16
    }

    pub fn amount_food_eaten(&self) -> u16 {
        match self.diet {
            Diet::Food { food: _, period } => {
                let size = self.maximum_size();
                let per_feed = if self.greedy { (4 * size) / 3 } else { size };
                // TODO should this be a float?
                per_feed / period
            }
            _ => 0,
        }
    }

    pub fn constraints(&self) -> Vec<Constraint> {
        let mut result = Vec::new();

        result.push(Constraint::Temperature(self.habitat.temperature));
        result.push(Constraint::Quality(self.habitat.minimum_quality));

        if let Some(s) = self.shoaling {
            result.push(Constraint::Shoaler(s));
        }

        if let Some(Fighting::Wimp) = self.fighting {
            result.push(Constraint::NoBully);
        }

        if let Some(l) = self.needs.light {
            result.push(Constraint::Lighting(l));
        }

        if let Some(c) = self.cohabitation {
            result.push(Constraint::Cohabitation(c));
        }

        if self.habitat.active_swimmer {
            result.push(Constraint::TankSize(self.minimum_needed_tank_size()));
        }

        if let Some(t) = self.habitat.interior {
            result.push(Constraint::Interior(t));
        }

        for p in &self.predation {
            result.push(Constraint::Predator {
                prey: p.clone(),
                size: self.predation_size(),
            });
        }

        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PreyType {
    Fish,
    Starfish,
    Crustacean,
    StonyCoral,
    SoftCoral,
    Clam,
    Gorgonian,
    Anemone,
}

impl PreyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreyType::Fish => "fish",
            PreyType::Starfish => "starfish",
            PreyType::Crustacean => "crustacean",
            PreyType::StonyCoral => "stonyCoral",
            PreyType::SoftCoral => "softCoral",
            PreyType::Clam => "clam",
            PreyType::Gorgonian => "gorgonian",
            PreyType::Anemone => "anemone",
        }
    }

    pub fn from_str(s: &str) -> Option<PreyType> {
        match s {
            "fish" => Some(PreyType::Fish),
            "starfish" => Some(PreyType::Starfish),
            "crustacean" => Some(PreyType::Crustacean),
            "stonyCoral" => Some(PreyType::StonyCoral),
            "softCoral" => Some(PreyType::SoftCoral),
            "clam" => Some(PreyType::Clam),
            "gorgonian" => Some(PreyType::Gorgonian),
            "anemone" => Some(PreyType::Anemone),
            _ => None,
        }
    }
}

as_str_display!(PreyType);

#[derive(Debug, Clone, PartialEq)]
pub enum Diet {
    Food { food: String, period: u16 },
    Scavenger,
    DoesNotEat,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Size {
    pub stages: Vec<Stage>,
    pub final_size: u16,
    pub armored: bool,
    pub immobile: bool,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Stage {
    pub size: u16,
    pub duration: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Habitat {
    pub minimum_quality: u8,
    pub temperature: Temperature,
    pub interior: Option<Interior>,
    pub active_swimmer: bool,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Needs {
    pub plants: Option<Need>,
    pub rocks: Option<Need>,
    pub caves: Option<u8>,
    pub light: Option<Need>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Need {
    Loves(u8),
    Dislikes,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Fighting {
    Wimp,
    Bully,
}

impl Fighting {
    pub fn as_str(&self) -> &'static str {
        match self {
            Fighting::Wimp => "wimp",
            Fighting::Bully => "bully",
        }
    }
}

as_str_display!(Fighting);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cohabitation {
    OnlyCongeners,
    NoConspecifics,
    NoCongeners,
    NoFoodCompetitors,
}

impl Cohabitation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Cohabitation::OnlyCongeners => "only-congeners",
            Cohabitation::NoConspecifics => "no-conspecifics",
            Cohabitation::NoCongeners => "no-congeners",
            Cohabitation::NoFoodCompetitors => "no-food-competitors",
        }
    }
}

as_str_display!(Cohabitation);

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::tank::*;

    pub fn test_species<S: Into<String>>(id: S) -> Species {
        Species {
            id: id.into(),
            genus: "fish".to_string(),
            prey_type: PreyType::Fish,
            size: Size {
                stages: Vec::new(),
                final_size: 5,
                immobile: false,
                armored: false,
            },
            habitat: Habitat {
                temperature: Temperature::Warm,
                minimum_quality: 55,
                active_swimmer: false,
                interior: None,
            },
            needs: Needs {
                plants: None,
                rocks: None,
                caves: None,
                light: None,
            },
            diet: Diet::DoesNotEat,
            greedy: false,
            shoaling: None,
            fighting: None,
            cohabitation: None,
            predation: Vec::new(),
        }
    }
}
