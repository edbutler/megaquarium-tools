use crate::tank::Environment;

#[derive(Debug)]
pub struct Species {
    pub id: String,
    pub kind: String,
    pub environment: Environment,
}
