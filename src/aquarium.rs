use crate::animal::*;
use crate::tank::*;

pub struct Aquarium<'a> {
    pub exhibits: Vec<Exhibit<'a>>,
}

pub struct Exhibit<'a> {
    pub tank: Tank,
    pub animals: Vec<Animal<'a>>,
}
