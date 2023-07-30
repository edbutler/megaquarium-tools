use crate::rules::Constraint;
use crate::tank::Environment;
use crate::util::as_str_display;

#[derive(Debug)]
pub struct Animal<'a> {
    pub id: u64,
    pub species: &'a Species,
    pub age: u32,
}

impl Animal<'_> {
    pub fn description(&self) -> AnimalDesc {
        AnimalDesc {
            species: self.species.id.clone(),
            size: self.size(),
            constraints: self.species.constraints(),
        }
    }

    pub fn size(&self) -> u16 {
        // TODO ignoring age for now
        self.species.size.final_size
    }
}

#[derive(Debug)]
pub struct AnimalSpec<'a> {
    pub species: &'a Species,
    pub count: u16,
}

pub struct AnimalDesc {
    pub species: String,
    pub size: u16,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, PartialEq)]
pub struct Species {
    pub id: String,
    pub kind: String,
    pub immobile: bool,
    pub size: Size,
    pub environment: Environment,
    pub diet: Diet,
    pub shoaling: Option<u8>,
    pub fighting: Option<Fighting>,
    pub lighting: Option<Lighting>,
    pub cohabitation: Option<Cohabitation>,
    pub tank: TankNeeds,
    pub predation: Vec<String>,
}

impl Species {
    pub fn is_bully(&self) -> bool {
        self.fighting == Some(Fighting::Bully)
    }

    pub fn minimum_needed_size(&self) -> u16 {
        if self.immobile {
            0
        } else {
            let size = self.size.final_size;
            if self.tank.active_swimmer {
                6 * size
            } else {
                size
            }
        }
    }

    pub fn maximum_used_tank_capacity(&self) -> u16 {
        if self.immobile {
            0
        } else {
            self.size.final_size
        }
    }

    pub fn needs_light(&self) -> bool {
        match self.lighting {
            Some(Lighting::Requires(_)) => true,
            _ => false,
        }
    }

    pub fn constraints(&self) -> Vec<Constraint> {
        let size = self.size.final_size;

        let mut result = Vec::new();

        result.push(Constraint::Temperature(self.environment.temperature));
        result.push(Constraint::Quality(self.environment.quality));

        let mut food_kind: Option<String> = None;

        match &self.diet {
            Diet::DoesNotEat => (),
            Diet::Scavenger => result.push(Constraint::Scavenger),
            Diet::Food { food, period } => {
                food_kind = Some(food.clone());
                result.push(Constraint::NeedsFood {
                    kind: food.clone(),
                    daily_amount: size / period,
                });
            }
        }

        if let Some(s) = self.shoaling {
            result.push(Constraint::Shoaler(s));
        }

        match self.fighting {
            Some(Fighting::Wimp) => result.push(Constraint::NoBully),
            _ => (),
        }

        match self.lighting {
            Some(Lighting::Disallows) => result.push(Constraint::NoLight),
            Some(Lighting::Requires(x)) => result.push(Constraint::NeedsLight(x)),
            None => (),
        }

        if let Some(c) = self.cohabitation {
            let constraint = match c {
                Cohabitation::NoConspecifics => Constraint::NoSpecies(self.id.clone()),
                Cohabitation::NoCongeners => Constraint::NoGenus(self.kind.clone()),
                Cohabitation::OnlyCongeners => Constraint::OnlyGenus(self.kind.clone()),
                Cohabitation::NoFoodCompetitors => Constraint::NoFoodEaters(food_kind.unwrap()),
            };
            result.push(constraint);
        }

        if self.tank.active_swimmer {
            result.push(Constraint::TankSize(self.minimum_needed_size()));
        }

        if self.tank.rounded_tank {
            result.push(Constraint::RoundedTank);
        }

        for p in &self.predation {
            // number from https://steamcommunity.com/app/600480/discussions/0/3276824488724294545/
            let size = (0.4 * (size as f64)).floor() as u16;
            result.push(Constraint::Predator { kind: p.clone(), size });
        }

        result
    }
}

#[derive(Debug, PartialEq)]
pub enum Diet {
    Food { food: String, period: u16 },
    Scavenger,
    DoesNotEat,
}

#[derive(Debug, PartialEq)]
pub struct Size {
    pub stages: Vec<Stage>,
    pub final_size: u16,
    pub armored: bool,
}

#[derive(Debug, PartialEq)]
pub struct Stage {
    pub size: u16,
    pub duration: u16,
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

#[derive(Debug, PartialEq)]
pub enum Lighting {
    Requires(u8),
    Disallows,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cohabitation {
    NoConspecifics,
    NoCongeners,
    OnlyCongeners,
    NoFoodCompetitors,
}

#[derive(Debug, PartialEq)]
pub struct TankNeeds {
    pub rounded_tank: bool,
    pub active_swimmer: bool,
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::tank::*;

    pub fn test_species<S: Into<String>>(id: S) -> Species {
        Species {
            id: id.into(),
            kind: "fish".to_string(),
            immobile: false,
            size: Size {
                armored: false,
                stages: Vec::new(),
                final_size: 5,
            },
            environment: Environment {
                temperature: Temperature::Warm,
                salinity: Salinity::Salty,
                quality: 55,
            },
            diet: Diet::DoesNotEat,
            shoaling: None,
            fighting: None,
            lighting: None,
            cohabitation: None,
            tank: TankNeeds {
                rounded_tank: false,
                active_swimmer: false,
            },
            predation: Vec::new(),
        }
    }
}
