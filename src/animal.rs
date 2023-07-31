use crate::rules::Constraint;
use crate::tank::Environment;
use crate::util::as_str_display;

#[derive(Debug)]
pub struct Animal<'a> {
    pub id: u64,
    pub species: &'a Species,
    pub age: u32,
}

#[derive(Debug)]
pub struct AnimalSpec<'a> {
    pub species: &'a Species,
    pub count: u16,
}

#[derive(Debug, PartialEq)]
pub struct Species {
    pub id: String,
    pub genus: String,
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

    pub fn predation_size(&self) -> u16 {
        let size = self.size.final_size;
        // number from https://steamcommunity.com/app/600480/discussions/0/3276824488724294545/
        (0.4 * (size as f64)).floor() as u16
    }

    pub fn constraints(&self) -> Vec<Constraint> {
        let mut result = Vec::new();

        result.push(Constraint::Temperature(self.environment.temperature));
        result.push(Constraint::Quality(self.environment.quality));

        if let Some(s) = self.shoaling {
            result.push(Constraint::Shoaler(s));
        }

        if let Some(Fighting::Wimp) = self.fighting {
            result.push(Constraint::NoBully);
        }

        if let Some(l) = self.lighting {
            result.push(Constraint::Lighting(l));
        }

        if let Some(c) = self.cohabitation {
            result.push(Constraint::Cohabitation(c));
        }

        if self.tank.active_swimmer {
            result.push(Constraint::TankSize(self.minimum_needed_size()));
        }

        if self.tank.rounded_tank {
            result.push(Constraint::RoundedTank);
        }

        for p in &self.predation {
            result.push(Constraint::Predator { genus: p.clone(), size: self.predation_size() });
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Lighting {
    Requires(u8),
    Disallows,
}

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
            genus: "fish".to_string(),
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
                plants: 0,
                rocks: 0,
                caves: 0,
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
