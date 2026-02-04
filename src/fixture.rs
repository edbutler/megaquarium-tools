pub type FixtureId = u64;

#[derive(Debug)]
pub struct Fixture {
    pub id: FixtureId,
    pub model: String,
}

#[derive(Debug)]
pub struct FixtureRef<'a> {
    pub id: FixtureId,
    pub model: &'a FixtureModel,
}

#[derive(Debug)]
pub struct FixtureModel {
    pub id: String,
    pub light: Option<u8>,
    pub plants: Option<u8>,
    pub rocks: Option<u8>,
    pub caves: Option<u8>,
    pub bogwood: Option<u8>,
    pub flat_surfaces: Option<u8>,
    pub vertical_surfaces: Option<u8>,
    pub fluffy_foliage: Option<u8>,
}
