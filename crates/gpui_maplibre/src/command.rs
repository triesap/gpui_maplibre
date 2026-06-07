use crate::control::NativeControlKind;
use crate::ids::MapHandle;
use crate::ids::{ControlHandle, MarkerHandle, PopupHandle};
use crate::marker::MarkerOptions;
use crate::options::MapInitOptions;
use crate::popup::PopupOptions;
use crate::subscription::{
    LayerEventSubscription, MapEventSubscription, MarkerDragEventSubscription,
    PopupEventSubscription,
};
use crate::types::MapControlAnchor;
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
    AddSource {
        handle: MapHandle,
        source_id: String,
        source_spec: serde_json::Value,
    },
    #[serde(rename = "add_geojson_source")]
    AddGeoJsonSource {
        handle: MapHandle,
        source_id: String,
        geojson: serde_json::Value,
        promote_id: Option<String>,
    },
    #[serde(rename = "update_geojson_source")]
    UpdateGeoJsonSource {
        handle: MapHandle,
        source_id: String,
        geojson: serde_json::Value,
    },
    RemoveSource {
        handle: MapHandle,
        source_id: String,
    },
    AddLayer {
        handle: MapHandle,
        layer_id: String,
        layer_spec: serde_json::Value,
        before_id: Option<String>,
    },
    RemoveLayer {
        handle: MapHandle,
        layer_id: String,
    },
    SetLayoutProperty {
        handle: MapHandle,
        layer_id: String,
        property_name: String,
        value: serde_json::Value,
    },
    SetPaintProperty {
        handle: MapHandle,
        layer_id: String,
        property_name: String,
        value: serde_json::Value,
    },
    SetFilter {
        handle: MapHandle,
        layer_id: String,
        filter: Option<serde_json::Value>,
    },
    SetLayerZoomRange {
        handle: MapHandle,
        layer_id: String,
        min_zoom: Option<f64>,
        max_zoom: Option<f64>,
    },
    SetFeatureState {
        handle: MapHandle,
        source_id: String,
        source_layer: Option<String>,
        feature_id: serde_json::Value,
        state: serde_json::Value,
    },
    SetTerrain {
        handle: MapHandle,
        terrain: Option<serde_json::Value>,
    },
    SetFog {
        handle: MapHandle,
        fog: Option<serde_json::Value>,
    },
    SetLight {
        handle: MapHandle,
        light: Option<serde_json::Value>,
    },
    AddNativeControl {
        request_id: u64,
        handle: MapHandle,
        kind: NativeControlKind,
        anchor: Option<MapControlAnchor>,
        options: Option<serde_json::Value>,
    },
    RemoveNativeControl {
        control_handle: ControlHandle,
    },
    CreateMarker {
        request_id: u64,
        handle: MapHandle,
        options: MarkerOptions,
    },
    UpdateMarker {
        marker_handle: MarkerHandle,
        options: MarkerOptions,
    },
    RemoveMarker {
        marker_handle: MarkerHandle,
    },
    CreatePopup {
        request_id: u64,
        handle: MapHandle,
        options: PopupOptions,
    },
    UpdatePopup {
        popup_handle: PopupHandle,
        options: PopupOptions,
    },
    RemovePopup {
        popup_handle: PopupHandle,
    },
    SubscribeMapEvents {
        handle: MapHandle,
        subscription: MapEventSubscription,
    },
    UnsubscribeMapEvents {
        handle: MapHandle,
    },
    SubscribeLayerEvents {
        handle: MapHandle,
        subscription: LayerEventSubscription,
    },
    UnsubscribeLayerEvents {
        handle: MapHandle,
        layer_id: String,
    },
    SubscribeMarkerDragEvents {
        marker_handle: MarkerHandle,
        subscription: MarkerDragEventSubscription,
    },
    UnsubscribeMarkerDragEvents {
        marker_handle: MarkerHandle,
    },
    SubscribePopupEvents {
        popup_handle: PopupHandle,
        subscription: PopupEventSubscription,
    },
    UnsubscribePopupEvents {
        popup_handle: PopupHandle,
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

    #[test]
    fn command_sources_serialize_payloads() {
        assert_eq!(
            serde_json::to_value(MapCommand::AddSource {
                handle: MapHandle(1),
                source_id: "tiles".to_owned(),
                source_spec: json!({"type": "vector", "url": "mapbox://tiles"}),
            })
            .unwrap(),
            json!({
                "type": "add_source",
                "handle": 1,
                "source_id": "tiles",
                "source_spec": {"type": "vector", "url": "mapbox://tiles"}
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::AddGeoJsonSource {
                handle: MapHandle(1),
                source_id: "places".to_owned(),
                geojson: json!({"type": "FeatureCollection", "features": []}),
                promote_id: Some("id".to_owned()),
            })
            .unwrap(),
            json!({
                "type": "add_geojson_source",
                "handle": 1,
                "source_id": "places",
                "geojson": {"type": "FeatureCollection", "features": []},
                "promote_id": "id"
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::UpdateGeoJsonSource {
                handle: MapHandle(1),
                source_id: "places".to_owned(),
                geojson: json!({"type": "FeatureCollection", "features": []}),
            })
            .unwrap(),
            json!({
                "type": "update_geojson_source",
                "handle": 1,
                "source_id": "places",
                "geojson": {"type": "FeatureCollection", "features": []}
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::RemoveSource {
                handle: MapHandle(1),
                source_id: "places".to_owned(),
            })
            .unwrap(),
            json!({
                "type": "remove_source",
                "handle": 1,
                "source_id": "places"
            })
        );
    }

    #[test]
    fn command_layers_serialize_payloads() {
        assert_eq!(
            serde_json::to_value(MapCommand::AddLayer {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
                layer_spec: json!({"id": "places-fill", "type": "fill", "source": "places"}),
                before_id: Some("labels".to_owned()),
            })
            .unwrap(),
            json!({
                "type": "add_layer",
                "handle": 1,
                "layer_id": "places-fill",
                "layer_spec": {"id": "places-fill", "type": "fill", "source": "places"},
                "before_id": "labels"
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::RemoveLayer {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
            })
            .unwrap(),
            json!({
                "type": "remove_layer",
                "handle": 1,
                "layer_id": "places-fill"
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetLayoutProperty {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
                property_name: "visibility".to_owned(),
                value: json!("none"),
            })
            .unwrap(),
            json!({
                "type": "set_layout_property",
                "handle": 1,
                "layer_id": "places-fill",
                "property_name": "visibility",
                "value": "none"
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetPaintProperty {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
                property_name: "fill-color".to_owned(),
                value: json!("#ffcc00"),
            })
            .unwrap(),
            json!({
                "type": "set_paint_property",
                "handle": 1,
                "layer_id": "places-fill",
                "property_name": "fill-color",
                "value": "#ffcc00"
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetFilter {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
                filter: None,
            })
            .unwrap(),
            json!({
                "type": "set_filter",
                "handle": 1,
                "layer_id": "places-fill",
                "filter": null
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetLayerZoomRange {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
                min_zoom: Some(3.0),
                max_zoom: None,
            })
            .unwrap(),
            json!({
                "type": "set_layer_zoom_range",
                "handle": 1,
                "layer_id": "places-fill",
                "min_zoom": 3.0,
                "max_zoom": null
            })
        );
    }

    #[test]
    fn command_feature_state_serializes_payloads() {
        assert_eq!(
            serde_json::to_value(MapCommand::SetFeatureState {
                handle: MapHandle(1),
                source_id: "places".to_owned(),
                source_layer: Some("settlements".to_owned()),
                feature_id: json!("place-1"),
                state: json!({"selected": true}),
            })
            .unwrap(),
            json!({
                "type": "set_feature_state",
                "handle": 1,
                "source_id": "places",
                "source_layer": "settlements",
                "feature_id": "place-1",
                "state": {"selected": true}
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetTerrain {
                handle: MapHandle(1),
                terrain: None,
            })
            .unwrap(),
            json!({
                "type": "set_terrain",
                "handle": 1,
                "terrain": null
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetFog {
                handle: MapHandle(1),
                fog: Some(json!({"range": [0.5, 10.0]})),
            })
            .unwrap(),
            json!({
                "type": "set_fog",
                "handle": 1,
                "fog": {"range": [0.5, 10.0]}
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SetLight {
                handle: MapHandle(1),
                light: None,
            })
            .unwrap(),
            json!({
                "type": "set_light",
                "handle": 1,
                "light": null
            })
        );
    }

    #[test]
    fn command_controls_serializes_payloads() {
        assert_eq!(
            serde_json::to_value(MapCommand::AddNativeControl {
                request_id: 42,
                handle: MapHandle(1),
                kind: NativeControlKind::Navigation,
                anchor: Some(crate::MapControlAnchor::TopRight),
                options: Some(json!({"showCompass": true})),
            })
            .unwrap(),
            json!({
                "type": "add_native_control",
                "request_id": 42,
                "handle": 1,
                "kind": "navigation",
                "anchor": "top_right",
                "options": {"showCompass": true}
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::RemoveNativeControl {
                control_handle: ControlHandle(7),
            })
            .unwrap(),
            json!({
                "type": "remove_native_control",
                "control_handle": 7
            })
        );
    }

    #[test]
    fn command_markers_serializes_payloads() {
        let options = MarkerOptions {
            lng: -123.1,
            lat: 49.2,
            draggable: true,
            anchor: Some("bottom".to_owned()),
            offset_x: Some(4.0),
            offset_y: Some(-8.0),
            rotation: Some(45.0),
        };

        assert_eq!(
            serde_json::to_value(MapCommand::CreateMarker {
                request_id: 100,
                handle: MapHandle(1),
                options: options.clone(),
            })
            .unwrap(),
            json!({
                "type": "create_marker",
                "request_id": 100,
                "handle": 1,
                "options": {
                    "lng": -123.1,
                    "lat": 49.2,
                    "draggable": true,
                    "anchor": "bottom",
                    "offset_x": 4.0,
                    "offset_y": -8.0,
                    "rotation": 45.0
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::UpdateMarker {
                marker_handle: MarkerHandle(5),
                options,
            })
            .unwrap(),
            json!({
                "type": "update_marker",
                "marker_handle": 5,
                "options": {
                    "lng": -123.1,
                    "lat": 49.2,
                    "draggable": true,
                    "anchor": "bottom",
                    "offset_x": 4.0,
                    "offset_y": -8.0,
                    "rotation": 45.0
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::RemoveMarker {
                marker_handle: MarkerHandle(5),
            })
            .unwrap(),
            json!({
                "type": "remove_marker",
                "marker_handle": 5
            })
        );
    }

    #[test]
    fn command_popups_serializes_payloads() {
        let options = PopupOptions {
            lng: -123.1,
            lat: 49.2,
            content: crate::PopupContent::Text("Hello map".to_owned()),
            close_button: Some(true),
            close_on_click: Some(false),
            anchor: Some("top".to_owned()),
            offset_x: None,
            offset_y: Some(12.0),
            max_width: Some(320.0),
        };

        assert_eq!(
            serde_json::to_value(MapCommand::CreatePopup {
                request_id: 101,
                handle: MapHandle(1),
                options: options.clone(),
            })
            .unwrap(),
            json!({
                "type": "create_popup",
                "request_id": 101,
                "handle": 1,
                "options": {
                    "lng": -123.1,
                    "lat": 49.2,
                    "content": {"kind": "text", "value": "Hello map"},
                    "close_button": true,
                    "close_on_click": false,
                    "anchor": "top",
                    "offset_x": null,
                    "offset_y": 12.0,
                    "max_width": 320.0
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::UpdatePopup {
                popup_handle: PopupHandle(6),
                options,
            })
            .unwrap(),
            json!({
                "type": "update_popup",
                "popup_handle": 6,
                "options": {
                    "lng": -123.1,
                    "lat": 49.2,
                    "content": {"kind": "text", "value": "Hello map"},
                    "close_button": true,
                    "close_on_click": false,
                    "anchor": "top",
                    "offset_x": null,
                    "offset_y": 12.0,
                    "max_width": 320.0
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::RemovePopup {
                popup_handle: PopupHandle(6),
            })
            .unwrap(),
            json!({
                "type": "remove_popup",
                "popup_handle": 6
            })
        );
    }

    #[test]
    fn command_subscriptions_serializes_payloads() {
        assert_eq!(
            serde_json::to_value(MapCommand::SubscribeMapEvents {
                handle: MapHandle(1),
                subscription: MapEventSubscription {
                    kinds: vec![crate::MapEventKind::MoveEnd, crate::MapEventKind::Error],
                    throttle_ms: Some(100),
                },
            })
            .unwrap(),
            json!({
                "type": "subscribe_map_events",
                "handle": 1,
                "subscription": {
                    "kinds": ["move_end", "error"],
                    "throttle_ms": 100
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SubscribeLayerEvents {
                handle: MapHandle(1),
                subscription: LayerEventSubscription {
                    layer_id: "places-fill".to_owned(),
                    kinds: vec![crate::LayerEventKind::Click],
                    throttle_ms: None,
                },
            })
            .unwrap(),
            json!({
                "type": "subscribe_layer_events",
                "handle": 1,
                "subscription": {
                    "layer_id": "places-fill",
                    "kinds": ["click"],
                    "throttle_ms": null
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SubscribeMarkerDragEvents {
                marker_handle: MarkerHandle(5),
                subscription: MarkerDragEventSubscription {
                    kinds: vec![
                        crate::MarkerDragEventKind::DragStart,
                        crate::MarkerDragEventKind::DragEnd
                    ],
                    throttle_ms: Some(50),
                },
            })
            .unwrap(),
            json!({
                "type": "subscribe_marker_drag_events",
                "marker_handle": 5,
                "subscription": {
                    "kinds": ["drag_start", "drag_end"],
                    "throttle_ms": 50
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::SubscribePopupEvents {
                popup_handle: PopupHandle(6),
                subscription: PopupEventSubscription {
                    kinds: vec![
                        crate::PopupLifecycleEventKind::Open,
                        crate::PopupLifecycleEventKind::Close
                    ],
                },
            })
            .unwrap(),
            json!({
                "type": "subscribe_popup_events",
                "popup_handle": 6,
                "subscription": {
                    "kinds": ["open", "close"]
                }
            })
        );

        assert_eq!(
            serde_json::to_value(MapCommand::UnsubscribeLayerEvents {
                handle: MapHandle(1),
                layer_id: "places-fill".to_owned(),
            })
            .unwrap(),
            json!({
                "type": "unsubscribe_layer_events",
                "handle": 1,
                "layer_id": "places-fill"
            })
        );
    }
}
