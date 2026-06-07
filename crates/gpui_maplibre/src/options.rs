use crate::types::MapControlAnchor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeControlOptions {
    pub navigation: Option<MapControlAnchor>,
    pub scale: Option<MapControlAnchor>,
    pub fullscreen: Option<MapControlAnchor>,
    pub geolocate: Option<MapControlAnchor>,
    pub attribution: Option<MapControlAnchor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapInitOptions {
    pub style_url: String,
    pub center_lng: f64,
    pub center_lat: f64,
    pub zoom: f64,
    pub min_zoom: Option<f64>,
    pub max_zoom: Option<f64>,
    pub min_pitch: Option<f64>,
    pub max_pitch: Option<f64>,
    pub bounds: Option<[f64; 4]>,
    pub max_bounds: Option<[f64; 4]>,
    pub pitch: Option<f64>,
    pub bearing: Option<f64>,
    pub bearing_snap: Option<f64>,
    pub projection: Option<String>,
    pub render_world_copies: Option<bool>,
    pub drag_pan: Option<bool>,
    pub drag_rotate: Option<bool>,
    pub pitch_with_rotate: Option<bool>,
    pub zoom_on_double_click: Option<bool>,
    pub cooperative_gestures: Option<bool>,
    pub preserve_drawing_buffer: Option<bool>,
    pub around_center: Option<bool>,
    pub interactive: Option<bool>,
    pub attribution_control: Option<bool>,
    pub antialias: Option<bool>,
    pub native_controls: Option<NativeControlOptions>,
}

impl Default for MapInitOptions {
    fn default() -> Self {
        Self {
            style_url: "https://demotiles.maplibre.org/style.json".to_owned(),
            center_lng: 0.0,
            center_lat: 0.0,
            zoom: 2.0,
            min_zoom: None,
            max_zoom: None,
            min_pitch: None,
            max_pitch: None,
            bounds: None,
            max_bounds: None,
            pitch: None,
            bearing: None,
            bearing_snap: None,
            projection: None,
            render_world_copies: None,
            drag_pan: None,
            drag_rotate: None,
            pitch_with_rotate: None,
            zoom_on_double_click: None,
            cooperative_gestures: None,
            preserve_drawing_buffer: None,
            around_center: None,
            interactive: None,
            attribution_control: None,
            antialias: None,
            native_controls: None,
        }
    }
}

impl MapInitOptions {
    pub fn with_style_url(mut self, style_url: impl Into<String>) -> Self {
        self.style_url = style_url.into();
        self
    }

    pub fn with_center(mut self, lng: f64, lat: f64) -> Self {
        self.center_lng = lng;
        self.center_lat = lat;
        self
    }

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn with_native_controls(mut self, native_controls: NativeControlOptions) -> Self {
        self.native_controls = Some(native_controls);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn map_init_options_roundtrip() {
        let options = MapInitOptions {
            style_url: "https://demotiles.maplibre.org/style.json".to_owned(),
            center_lng: 15.0,
            center_lat: 60.0,
            zoom: 6.0,
            min_zoom: Some(2.0),
            max_zoom: Some(18.0),
            min_pitch: Some(0.0),
            max_pitch: Some(60.0),
            bounds: Some([-179.0, -80.0, 179.0, 80.0]),
            max_bounds: Some([-160.0, -70.0, 160.0, 70.0]),
            pitch: Some(35.0),
            bearing: Some(15.0),
            bearing_snap: Some(7.0),
            projection: Some("mercator".to_owned()),
            render_world_copies: Some(true),
            drag_pan: Some(true),
            drag_rotate: Some(true),
            pitch_with_rotate: Some(true),
            zoom_on_double_click: Some(true),
            cooperative_gestures: Some(false),
            preserve_drawing_buffer: Some(false),
            around_center: Some(false),
            interactive: Some(true),
            attribution_control: Some(false),
            antialias: Some(true),
            native_controls: Some(NativeControlOptions {
                navigation: Some(MapControlAnchor::TopRight),
                scale: Some(MapControlAnchor::BottomLeft),
                fullscreen: None,
                geolocate: None,
                attribution: Some(MapControlAnchor::BottomRight),
            }),
        };

        let encoded = serde_json::to_string(&options).unwrap();
        let decoded: MapInitOptions = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, options);
    }

    #[test]
    fn map_init_options_default_serializes_expected_fields() {
        let encoded = serde_json::to_value(MapInitOptions::default()).unwrap();

        assert_eq!(
            encoded,
            json!({
                "style_url": "https://demotiles.maplibre.org/style.json",
                "center_lng": 0.0,
                "center_lat": 0.0,
                "zoom": 2.0,
                "min_zoom": null,
                "max_zoom": null,
                "min_pitch": null,
                "max_pitch": null,
                "bounds": null,
                "max_bounds": null,
                "pitch": null,
                "bearing": null,
                "bearing_snap": null,
                "projection": null,
                "render_world_copies": null,
                "drag_pan": null,
                "drag_rotate": null,
                "pitch_with_rotate": null,
                "zoom_on_double_click": null,
                "cooperative_gestures": null,
                "preserve_drawing_buffer": null,
                "around_center": null,
                "interactive": null,
                "attribution_control": null,
                "antialias": null,
                "native_controls": null
            })
        );
    }

    #[test]
    fn native_controls_default_hidden() {
        let options = MapInitOptions::default();

        assert!(options.native_controls.is_none());
        assert_eq!(
            NativeControlOptions::default(),
            NativeControlOptions {
                navigation: None,
                scale: None,
                fullscreen: None,
                geolocate: None,
                attribution: None,
            }
        );
    }

    #[test]
    fn map_init_options_builder_helpers_remain_field_transparent() {
        let options = MapInitOptions::default()
            .with_style_url("maplibre://styles/basic")
            .with_center(-123.1, 49.2)
            .with_zoom(11.5)
            .with_native_controls(NativeControlOptions {
                navigation: Some(MapControlAnchor::TopLeft),
                ..NativeControlOptions::default()
            });

        assert_eq!(options.style_url, "maplibre://styles/basic");
        assert_eq!(options.center_lng, -123.1);
        assert_eq!(options.center_lat, 49.2);
        assert_eq!(options.zoom, 11.5);
        assert_eq!(
            options.native_controls.unwrap().navigation,
            Some(MapControlAnchor::TopLeft)
        );
    }
}
