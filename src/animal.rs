use crate::rules::Constraint;
use crate::tank::Environment;

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
            constraints: self.constraints(),
        }
    }

    pub fn size(&self) -> u16 {
        // TODO ignoring age for now
        self.species.size.final_size
    }

    pub fn constraints(&self) -> Vec<Constraint> {
        let spec = self.species;
        let mut result = Vec::new();

        result.push(Constraint::Temperature(spec.environment.temperature));
        result.push(Constraint::Quality(spec.environment.quality));

        let mut food_kind: Option<String> = None;

        match &spec.diet {
            Diet::DoesNotEat => (),
            Diet::Scavenger => result.push(Constraint::Scavenger),
            Diet::Food { food, period } => {
                food_kind = Some(food.clone());
                result.push(Constraint::NeedsFood {
                    kind: food.clone(),
                    daily_amount: self.size() / period,
                });
            }
        }

        if let Some(s) = spec.shoaling {
            result.push(Constraint::Shoaler(s));
        }

        match spec.fighting {
            Some(Fighting::Wimp) => result.push(Constraint::NoBully),
            Some(Fighting::Bully) => result.push(Constraint::IsBully),
            None => (),
        }

        match spec.lighting {
            Some(Lighting::Disallows) => result.push(Constraint::NoLight),
            Some(Lighting::Requires(x)) => result.push(Constraint::NeedsLight(x)),
            None => (),
        }

        if let Some(c) = spec.cohabitation {
            let constraint = match c {
                Cohabitation::NoConspecifics => Constraint::NoSpecies(spec.id.clone()),
                Cohabitation::NoCongeners => Constraint::NoGenus(spec.kind.clone()),
                Cohabitation::OnlyCongeners => Constraint::OnlyGenus(spec.kind.clone()),
                Cohabitation::NoFoodCompetitors => Constraint::NoFoodEaters(food_kind.unwrap()),
            };
            result.push(constraint);
        }

        if spec.tank.active_swimmer {
            result.push(Constraint::TankSize(6 * self.size()));
        }

        if spec.tank.rounded_tank {
            result.push(Constraint::RoundedTank);
        }

        for p in &spec.predation {
            // number from https://steamcommunity.com/app/600480/discussions/0/3276824488724294545/
            let size = (0.4 * (self.size() as f64)).floor() as u16;
            result.push(Constraint::Predator { kind: p.clone(), size });
        }

        result
    }
}

#[derive(Debug)]
pub struct AnimalSpec {
    pub species: String,
    pub count: u64,
}

impl std::fmt::Display for AnimalSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.count, self.species)
    }
}

pub struct AnimalDesc {
    pub species: String,
    pub size: u16,
    pub constraints: Vec<Constraint>,
}

impl std::fmt::Display for AnimalDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(animal\n")?;
        write!(f, "  #:species {}\n", self.species)?;
        write!(f, "  #:size {}\n", self.size)?;
        write!(f, "  #:constraints (")?;
        for c in &self.constraints {
            write!(f, "\n    {}", c)?;
        }
        write!(f, "))\n")?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Species {
    pub id: String,
    pub kind: String,
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

    pub fn test_species<S : Into<String>>(id: S) -> Species {
        Species {
            id: id.into(),
            kind: "fish".to_string(),
            size: Size { armored: false, stages: Vec::new(), final_size: 5 },
            environment: Environment { temperature: Temperature::Warm, salinity: Salinity::Salty, quality: 55 },
            diet: Diet::DoesNotEat,
            shoaling: None,
            fighting: None,
            lighting: None,
            cohabitation: None,
            tank: TankNeeds { rounded_tank: false, active_swimmer: false },
            predation: Vec::new(),
        }
    }
}
