use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LngLat {
    pub lng: f64,
    pub lat: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapControlAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_primitives_roundtrip() {
        let coordinate = LngLat {
            lng: -123.1207,
            lat: 49.2827,
        };
        let bounds = Bounds {
            west: -124.0,
            south: 48.0,
            east: -122.0,
            north: 50.0,
        };

        let coordinate_json = serde_json::to_string(&coordinate).unwrap();
        let bounds_json = serde_json::to_string(&bounds).unwrap();

        assert_eq!(
            serde_json::from_str::<LngLat>(&coordinate_json).unwrap(),
            coordinate
        );
        assert_eq!(
            serde_json::from_str::<Bounds>(&bounds_json).unwrap(),
            bounds
        );
    }

    #[test]
    fn map_control_anchor_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&MapControlAnchor::TopRight).unwrap(),
            "\"top_right\""
        );
        assert_eq!(
            serde_json::from_str::<MapControlAnchor>("\"bottom_left\"").unwrap(),
            MapControlAnchor::BottomLeft
        );
    }
}
