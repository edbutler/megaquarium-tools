#[derive(Debug)]
pub struct Tank {}

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

#[derive(Debug)]
pub struct Environment {
    pub temperature: Temperature,
    pub salinity: Salinity,
    pub quality: u8,
}
