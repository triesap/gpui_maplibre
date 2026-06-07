use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    GeoJson,
    Vector,
    Raster,
    RasterDem,
    Image,
    Video,
    Canvas,
}

impl SourceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GeoJson => "geojson",
            Self::Vector => "vector",
            Self::Raster => "raster",
            Self::RasterDem => "raster-dem",
            Self::Image => "image",
            Self::Video => "video",
            Self::Canvas => "canvas",
        }
    }
}

pub fn geojson_source(geojson: Value) -> Value {
    json!({
        "type": SourceType::GeoJson.as_str(),
        "data": geojson
    })
}

pub fn vector_source(url: impl Into<String>) -> Value {
    json!({
        "type": SourceType::Vector.as_str(),
        "url": url.into()
    })
}

pub fn raster_source(tiles: Vec<String>, tile_size: Option<u32>) -> Value {
    json!({
        "type": SourceType::Raster.as_str(),
        "tiles": tiles,
        "tileSize": tile_size
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn source_type_helpers_match_maplibre_style_spec_names() {
        assert_eq!(SourceType::GeoJson.as_str(), "geojson");
        assert_eq!(SourceType::RasterDem.as_str(), "raster-dem");

        assert_eq!(
            geojson_source(json!({
                "type": "FeatureCollection",
                "features": []
            })),
            json!({
                "type": "geojson",
                "data": {
                    "type": "FeatureCollection",
                    "features": []
                }
            })
        );

        assert_eq!(
            vector_source("mapbox://tiles"),
            json!({
                "type": "vector",
                "url": "mapbox://tiles"
            })
        );

        assert_eq!(
            raster_source(
                vec!["https://tiles.example/{z}/{x}/{y}.png".to_owned()],
                Some(256)
            ),
            json!({
                "type": "raster",
                "tiles": ["https://tiles.example/{z}/{x}/{y}.png"],
                "tileSize": 256
            })
        );
    }
}
