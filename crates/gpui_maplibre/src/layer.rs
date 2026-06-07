use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerType {
    Fill,
    Line,
    Symbol,
    Circle,
    Heatmap,
    FillExtrusion,
    Raster,
    Hillshade,
    Background,
}

impl LayerType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fill => "fill",
            Self::Line => "line",
            Self::Symbol => "symbol",
            Self::Circle => "circle",
            Self::Heatmap => "heatmap",
            Self::FillExtrusion => "fill-extrusion",
            Self::Raster => "raster",
            Self::Hillshade => "hillshade",
            Self::Background => "background",
        }
    }
}

pub fn sourced_layer(layer_type: LayerType, source_id: impl AsRef<str>) -> Value {
    json!({
        "type": layer_type.as_str(),
        "source": source_id.as_ref()
    })
}

pub fn background_layer() -> Value {
    json!({
        "type": LayerType::Background.as_str()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn layer_type_helpers_match_maplibre_style_spec_names() {
        assert_eq!(LayerType::FillExtrusion.as_str(), "fill-extrusion");
        assert_eq!(LayerType::Background.as_str(), "background");

        assert_eq!(
            sourced_layer(LayerType::Circle, "places"),
            json!({
                "type": "circle",
                "source": "places"
            })
        );

        assert_eq!(
            background_layer(),
            json!({
                "type": "background"
            })
        );
    }
}
