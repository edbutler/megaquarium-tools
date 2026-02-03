
pub type DecorationId = u64;

pub struct Decoration {
    pub id: DecorationId,
    pub model: String,
}

pub struct DecorationModel {
    pub id: String,
    pub light: Option<u16>,
    pub plants: Option<u16>,
    pub rocks: Option<u16>,
    pub caves: Option<u16>,
    pub bogwood: Option<u16>,
    pub flat_surfaces: Option<u16>,
    pub vertical_surfaces: Option<u16>,
    pub fluffy_foliage: Option<u16>,
}
