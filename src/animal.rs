use crate::tank::Environment;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Diet {
    Food { food: String, period: u8 },
    Scavenger,
    DoesNotEat,
}

#[derive(Debug)]
pub struct Size {
    pub stages: Vec<Stage>,
    pub final_size: u8,
    pub armored: bool,
}

#[derive(Debug)]
pub struct Stage {
    pub size: u8,
    pub duration: u8,
}

#[derive(Debug, Copy, Clone)]
pub enum Fighting {
    Wimp,
    Bully,
}

#[derive(Debug)]
pub enum Lighting {
    Requires(u8),
    Disallows,
}

#[derive(Debug, Copy, Clone)]
pub enum Cohabitation {
    NoConspecifics,
    NoCongeners,
    OnlyCongeners,
    NoFoodCompetitors,
}

#[derive(Debug)]
pub struct TankNeeds {
    pub rounded_tank: bool,
    pub active_swimmer: bool,
}
