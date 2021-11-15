use crate::tank::Environment;

#[derive(Debug)]
pub struct Species {
    pub id: String,
    pub kind: String,
    pub size: Size,
    pub environment: Environment,
    pub diet: Diet,
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
}

#[derive(Debug)]
pub struct Stage {
    pub size: u8,
    pub duration: u8,
}
