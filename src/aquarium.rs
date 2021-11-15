
use crate::tank::*;
use crate::animal::*;

pub struct Aquarium<'a> {
    pub exhibits: Vec<Exhibit<'a>>,
}

pub struct Exhibit<'a> {
    pub tank: Tank,
    pub animals: Animal<'a>,
}
