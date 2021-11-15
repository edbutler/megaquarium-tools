#[derive(Debug,Copy,Clone)]
pub struct Tank {
    pub id: u64,
}

pub struct TankModel {}

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
