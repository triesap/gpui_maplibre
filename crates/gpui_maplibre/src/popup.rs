use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PopupContent {
    Text(String),
    TrustedHtml(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PopupOptions {
    pub lng: f64,
    pub lat: f64,
    pub content: PopupContent,
    pub close_button: Option<bool>,
    pub close_on_click: Option<bool>,
    pub anchor: Option<String>,
    pub offset_x: Option<f64>,
    pub offset_y: Option<f64>,
    pub max_width: Option<f64>,
}
