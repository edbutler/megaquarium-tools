
pub type DecorationId = u64;

pub struct Decoration {
    pub id: DecorationId,
    pub model: String,
}

pub struct DecorationModel {
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
