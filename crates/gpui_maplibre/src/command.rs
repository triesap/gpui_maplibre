use crate::ids::MapHandle;
use crate::options::MapInitOptions;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapCommand {
    Init {
        options: MapInitOptions,
    },
    Destroy {
        handle: MapHandle,
    },
    Resize {
        handle: MapHandle,
    },
    SetStyle {
        handle: MapHandle,
        style_url: String,
    },
    FlyTo {
        handle: MapHandle,
        lng: f64,
        lat: f64,
        zoom: Option<f64>,
        duration_ms: Option<u32>,
    },
    JumpTo {
        handle: MapHandle,
        lng: f64,
        lat: f64,
        zoom: Option<f64>,
        bearing: Option<f64>,
        pitch: Option<f64>,
    },
    EaseTo {
        handle: MapHandle,
        lng: f64,
        lat: f64,
        zoom: Option<f64>,
        bearing: Option<f64>,
        pitch: Option<f64>,
        duration_ms: Option<u32>,
    },
    FitBounds {
        handle: MapHandle,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        padding: Option<f64>,
        duration_ms: Option<u32>,
        max_zoom: Option<f64>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn command_lifecycle_serializes() {
        assert_eq!(
            serde_json::to_value(MapCommand::Init {
                options: MapInitOptions::default(),
            })
            .unwrap(),
            json!({
                "type": "init",
                "options": MapInitOptions::default()
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::Destroy {
                handle: MapHandle(1),
            })
            .unwrap(),
            json!({
                "type": "destroy",
                "handle": 1
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::Resize {
                handle: MapHandle(1),
            })
            .unwrap(),
            json!({
                "type": "resize",
                "handle": 1
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetStyle {
                handle: MapHandle(1),
                style_url: "maplibre://styles/basic".to_owned(),
            })
            .unwrap(),
            json!({
                "type": "set_style",
                "handle": 1,
                "style_url": "maplibre://styles/basic"
            })
        );
    }

    #[test]
    fn command_camera_serializes() {
        assert_eq!(
            serde_json::to_value(MapCommand::FlyTo {
                handle: MapHandle(1),
                lng: -123.1,
                lat: 49.2,
                zoom: Some(11.0),
                duration_ms: Some(750),
            })
            .unwrap(),
            json!({
                "type": "fly_to",
                "handle": 1,
                "lng": -123.1,
                "lat": 49.2,
                "zoom": 11.0,
                "duration_ms": 750
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::JumpTo {
                handle: MapHandle(1),
                lng: -123.1,
                lat: 49.2,
                zoom: None,
                bearing: Some(15.0),
                pitch: Some(30.0),
            })
            .unwrap(),
            json!({
                "type": "jump_to",
                "handle": 1,
                "lng": -123.1,
                "lat": 49.2,
                "zoom": null,
                "bearing": 15.0,
                "pitch": 30.0
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::EaseTo {
                handle: MapHandle(1),
                lng: -123.1,
                lat: 49.2,
                zoom: Some(12.0),
                bearing: None,
                pitch: None,
                duration_ms: Some(500),
            })
            .unwrap(),
            json!({
                "type": "ease_to",
                "handle": 1,
                "lng": -123.1,
                "lat": 49.2,
                "zoom": 12.0,
                "bearing": null,
                "pitch": null,
                "duration_ms": 500
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::FitBounds {
                handle: MapHandle(1),
                west: -124.0,
                south: 48.0,
                east: -122.0,
                north: 50.0,
                padding: Some(24.0),
                duration_ms: None,
                max_zoom: Some(14.0),
            })
            .unwrap(),
            json!({
                "type": "fit_bounds",
                "handle": 1,
                "west": -124.0,
                "south": 48.0,
                "east": -122.0,
                "north": 50.0,
                "padding": 24.0,
                "duration_ms": null,
                "max_zoom": 14.0
            })
        );
    }
}
