use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkerOptions {
    pub lng: f64,
    pub lat: f64,
    pub draggable: bool,
    pub anchor: Option<String>,
    pub offset_x: Option<f64>,
    pub offset_y: Option<f64>,
    pub rotation: Option<f64>,
}
