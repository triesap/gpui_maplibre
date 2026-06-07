use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapEventKind {
    MoveStart,
    Move,
    MoveEnd,
    ZoomStart,
    Zoom,
    ZoomEnd,
    Idle,
    Resize,
    Render,
    StyleLoad,
    StyleData,
    SourceData,
    Data,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerEventKind {
    Click,
    DoubleClick,
    ContextMenu,
    MouseDown,
    MouseUp,
    MouseOver,
    MouseOut,
    MouseEnter,
    MouseMove,
    MouseLeave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerDragEventKind {
    DragStart,
    Drag,
    DragEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopupLifecycleEventKind {
    Open,
    Close,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapViewState {
    pub center_lng: f64,
    pub center_lat: f64,
    pub zoom: f64,
    pub bearing: f64,
    pub pitch: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapEvent {
    pub kind: MapEventKind,
    pub view: MapViewState,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureHit {
    pub layer_id: String,
    pub properties: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapClickEvent {
    pub lng: f64,
    pub lat: f64,
    pub screen_x: f64,
    pub screen_y: f64,
    pub features: Vec<FeatureHit>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerFeatureHit {
    pub layer_id: String,
    pub properties: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerEvent {
    pub kind: LayerEventKind,
    pub layer_id: String,
    pub lng: f64,
    pub lat: f64,
    pub screen_x: f64,
    pub screen_y: f64,
    pub features: Vec<LayerFeatureHit>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkerDragEvent {
    pub kind: MarkerDragEventKind,
    pub lng: f64,
    pub lat: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PopupLifecycleEvent {
    pub kind: PopupLifecycleEventKind,
    pub lng: f64,
    pub lat: f64,
}

#[cfg(test)]
mod events_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn event_roundtrip() {
        let event = MapEvent {
            kind: MapEventKind::Resize,
            view: MapViewState {
                center_lng: 18.06,
                center_lat: 59.33,
                zoom: 8.5,
                bearing: 15.0,
                pitch: 30.0,
            },
            message: None,
        };

        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: MapEvent = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, event);
    }

    #[test]
    fn map_click_event_roundtrip() {
        let event = MapClickEvent {
            lng: 11.2,
            lat: 58.9,
            screen_x: 256.0,
            screen_y: 384.0,
            features: vec![FeatureHit {
                layer_id: "points".to_owned(),
                properties: json!({"name":"harbor"}),
            }],
        };

        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: MapClickEvent = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, event);
    }

    #[test]
    fn layer_event_roundtrip() {
        let event = LayerEvent {
            kind: LayerEventKind::Click,
            layer_id: "places-fill".to_owned(),
            lng: 11.2,
            lat: 58.9,
            screen_x: 512.0,
            screen_y: 288.0,
            features: vec![LayerFeatureHit {
                layer_id: "places-fill".to_owned(),
                properties: json!({"id":"SE-123","selected":true}),
            }],
        };

        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: LayerEvent = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, event);
    }

    #[test]
    fn marker_drag_event_roundtrip() {
        let event = MarkerDragEvent {
            kind: MarkerDragEventKind::DragEnd,
            lng: 12.0,
            lat: 57.5,
        };

        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: MarkerDragEvent = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, event);
    }

    #[test]
    fn popup_lifecycle_event_roundtrip() {
        let event = PopupLifecycleEvent {
            kind: PopupLifecycleEventKind::Open,
            lng: 13.2,
            lat: 59.4,
        };

        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: PopupLifecycleEvent = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, event);
    }

    #[test]
    fn map_error_event_deserialize_with_message() {
        let payload = json!({
            "kind": "error",
            "view": {
                "center_lng": 11.0,
                "center_lat": 57.0,
                "zoom": 5.0,
                "bearing": 0.0,
                "pitch": 0.0
            },
            "message": "tile request failed"
        });

        let decoded: MapEvent = serde_json::from_value(payload).unwrap();

        assert_eq!(decoded.kind, MapEventKind::Error);
        assert_eq!(decoded.message.as_deref(), Some("tile request failed"));
    }

    #[test]
    fn event_kinds_use_snake_case() {
        assert_eq!(
            serde_json::to_string(&MapEventKind::MoveStart).unwrap(),
            "\"move_start\""
        );
        assert_eq!(
            serde_json::to_string(&LayerEventKind::DoubleClick).unwrap(),
            "\"double_click\""
        );
        assert_eq!(
            serde_json::to_string(&MarkerDragEventKind::DragEnd).unwrap(),
            "\"drag_end\""
        );
        assert_eq!(
            serde_json::to_string(&PopupLifecycleEventKind::Close).unwrap(),
            "\"close\""
        );
    }
}
