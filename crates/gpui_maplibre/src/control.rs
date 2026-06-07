use crate::types::MapControlAnchor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeControlKind {
    Navigation,
    Scale,
    Fullscreen,
    Geolocate,
    Attribution,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeControlOptions {
    pub kind: NativeControlKind,
    pub anchor: Option<MapControlAnchor>,
    pub options: Option<serde_json::Value>,
}
