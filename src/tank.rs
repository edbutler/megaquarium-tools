#[derive(Debug)]
pub struct Tank {
    pub id: String,
    pub environment: Environment,
    pub lighting: u8,
}

pub struct TankModel {

}

#[derive(Debug)]
pub struct Environment {
    pub temperature: Temperature,
    pub salinity: Salinity,
    pub quality: u8,
}

#[derive(Debug)]
pub enum Temperature {
    Warm,
    Cold,
}

#[derive(Debug)]
pub enum Salinity {
    Fresh,
    Salty,
}
