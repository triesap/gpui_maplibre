use crate::ids::{ControlHandle, MapHandle, MarkerHandle, PopupHandle};
use crate::{MapLibreError, Result};
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapLibreEvent {
    DomReady,
    Initialized {
        handle: MapHandle,
    },
    Ready {
        handle: MapHandle,
    },
    Click {
        lng: f64,
        lat: f64,
        screen_x: f64,
        screen_y: f64,
        features: Vec<FeatureHit>,
    },
    Map {
        handle: MapHandle,
        event: MapEvent,
    },
    Layer {
        handle: MapHandle,
        event: LayerEvent,
    },
    NativeControlCreated {
        request_id: u64,
        control_handle: ControlHandle,
    },
    MarkerCreated {
        request_id: u64,
        marker_handle: MarkerHandle,
    },
    MarkerDrag {
        marker_handle: MarkerHandle,
        event: MarkerDragEvent,
    },
    PopupCreated {
        request_id: u64,
        popup_handle: PopupHandle,
    },
    Popup {
        popup_handle: PopupHandle,
        event: PopupLifecycleEvent,
    },
    Error {
        context: String,
        message: String,
    },
}

impl MapLibreEvent {
    pub fn request_id(&self) -> Option<u64> {
        match self {
            Self::NativeControlCreated { request_id, .. }
            | Self::MarkerCreated { request_id, .. }
            | Self::PopupCreated { request_id, .. } => Some(*request_id),
            Self::DomReady
            | Self::Initialized { .. }
            | Self::Ready { .. }
            | Self::Click { .. }
            | Self::Map { .. }
            | Self::Layer { .. }
            | Self::MarkerDrag { .. }
            | Self::Popup { .. }
            | Self::Error { .. } => None,
        }
    }

    pub fn map_handle(&self) -> Option<MapHandle> {
        match self {
            Self::Initialized { handle } | Self::Ready { handle } => Some(*handle),
            Self::Map { handle, .. } | Self::Layer { handle, .. } => Some(*handle),
            Self::DomReady
            | Self::Click { .. }
            | Self::NativeControlCreated { .. }
            | Self::MarkerCreated { .. }
            | Self::MarkerDrag { .. }
            | Self::PopupCreated { .. }
            | Self::Popup { .. }
            | Self::Error { .. } => None,
        }
    }
}

pub fn parse_ipc_event(message: &str) -> Result<MapLibreEvent> {
    serde_json::from_str(message).map_err(MapLibreError::invalid_event)
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

    #[test]
    fn maplibre_event_deserializes_ipc_payloads() {
        let event = serde_json::from_str::<MapLibreEvent>(r#"{"type":"dom_ready"}"#).unwrap();
        assert_eq!(event, MapLibreEvent::DomReady);

        let event =
            serde_json::from_str::<MapLibreEvent>(r#"{"type":"initialized","handle":1}"#).unwrap();
        assert_eq!(
            event,
            MapLibreEvent::Initialized {
                handle: MapHandle(1),
            }
        );

        let event = serde_json::from_value::<MapLibreEvent>(json!({
            "type": "click",
            "lng": -123.1,
            "lat": 49.2,
            "screen_x": 100.0,
            "screen_y": 200.0,
            "features": [
                {"layer_id": "places", "properties": {"name": "Harbor"}}
            ]
        }))
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::Click {
                lng: -123.1,
                lat: 49.2,
                screen_x: 100.0,
                screen_y: 200.0,
                features: vec![FeatureHit {
                    layer_id: "places".to_owned(),
                    properties: json!({"name": "Harbor"}),
                }],
            }
        );
    }

    #[test]
    fn maplibre_event_wraps_nested_payloads() {
        let event = serde_json::from_value::<MapLibreEvent>(json!({
            "type": "map",
            "handle": 1,
            "event": {
                "kind": "move_end",
                "view": {
                    "center_lng": -123.1,
                    "center_lat": 49.2,
                    "zoom": 10.0,
                    "bearing": 0.0,
                    "pitch": 0.0
                }
            }
        }))
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::Map {
                handle: MapHandle(1),
                event: MapEvent {
                    kind: MapEventKind::MoveEnd,
                    view: MapViewState {
                        center_lng: -123.1,
                        center_lat: 49.2,
                        zoom: 10.0,
                        bearing: 0.0,
                        pitch: 0.0,
                    },
                    message: None,
                },
            }
        );

        let event = serde_json::from_value::<MapLibreEvent>(json!({
            "type": "layer",
            "handle": 1,
            "event": {
                "kind": "click",
                "layer_id": "places-fill",
                "lng": -123.1,
                "lat": 49.2,
                "screen_x": 100.0,
                "screen_y": 200.0,
                "features": []
            }
        }))
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::Layer {
                handle: MapHandle(1),
                event: LayerEvent {
                    kind: LayerEventKind::Click,
                    layer_id: "places-fill".to_owned(),
                    lng: -123.1,
                    lat: 49.2,
                    screen_x: 100.0,
                    screen_y: 200.0,
                    features: Vec::new(),
                },
            }
        );
    }

    #[test]
    fn maplibre_event_deserializes_handle_acknowledgements_and_errors() {
        let event = serde_json::from_str::<MapLibreEvent>(
            r#"{"type":"native_control_created","request_id":10,"control_handle":2}"#,
        )
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::NativeControlCreated {
                request_id: 10,
                control_handle: ControlHandle(2),
            }
        );

        let event = serde_json::from_str::<MapLibreEvent>(
            r#"{"type":"marker_created","request_id":11,"marker_handle":3}"#,
        )
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::MarkerCreated {
                request_id: 11,
                marker_handle: MarkerHandle(3),
            }
        );

        let event = serde_json::from_str::<MapLibreEvent>(
            r#"{"type":"popup_created","request_id":12,"popup_handle":4}"#,
        )
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::PopupCreated {
                request_id: 12,
                popup_handle: PopupHandle(4),
            }
        );

        let event = serde_json::from_str::<MapLibreEvent>(
            r#"{"type":"error","context":"add_layer","message":"duplicate id"}"#,
        )
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::Error {
                context: "add_layer".to_owned(),
                message: "duplicate id".to_owned(),
            }
        );
    }

    #[test]
    fn maplibre_event_deserializes_marker_and_popup_events() {
        let event = serde_json::from_value::<MapLibreEvent>(json!({
            "type": "marker_drag",
            "marker_handle": 3,
            "event": {
                "kind": "drag_end",
                "lng": -123.1,
                "lat": 49.2
            }
        }))
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::MarkerDrag {
                marker_handle: MarkerHandle(3),
                event: MarkerDragEvent {
                    kind: MarkerDragEventKind::DragEnd,
                    lng: -123.1,
                    lat: 49.2,
                },
            }
        );

        let event = serde_json::from_value::<MapLibreEvent>(json!({
            "type": "popup",
            "popup_handle": 4,
            "event": {
                "kind": "open",
                "lng": -123.1,
                "lat": 49.2
            }
        }))
        .unwrap();
        assert_eq!(
            event,
            MapLibreEvent::Popup {
                popup_handle: PopupHandle(4),
                event: PopupLifecycleEvent {
                    kind: PopupLifecycleEventKind::Open,
                    lng: -123.1,
                    lat: 49.2,
                },
            }
        );
    }

    #[test]
    fn ipc_event_parser_accepts_bridge_emitted_envelopes() {
        assert_eq!(
            parse_ipc_event(r#"{"type":"dom_ready"}"#).unwrap(),
            MapLibreEvent::DomReady
        );
        assert_eq!(
            parse_ipc_event(r#"{"type":"initialized","handle":1}"#).unwrap(),
            MapLibreEvent::Initialized {
                handle: MapHandle(1),
            }
        );
        assert_eq!(
            parse_ipc_event(
                r#"{"type":"native_control_created","request_id":10,"control_handle":7}"#
            )
            .unwrap(),
            MapLibreEvent::NativeControlCreated {
                request_id: 10,
                control_handle: ControlHandle(7),
            }
        );
        assert_eq!(
            parse_ipc_event(r#"{"type":"marker_created","request_id":11,"marker_handle":2}"#)
                .unwrap(),
            MapLibreEvent::MarkerCreated {
                request_id: 11,
                marker_handle: MarkerHandle(2),
            }
        );
        assert_eq!(
            parse_ipc_event(r#"{"type":"popup_created","request_id":12,"popup_handle":4}"#)
                .unwrap(),
            MapLibreEvent::PopupCreated {
                request_id: 12,
                popup_handle: PopupHandle(4),
            }
        );
        assert_eq!(
            parse_ipc_event(r#"{"type":"error","context":"dispatch","message":"bad command"}"#)
                .unwrap(),
            MapLibreEvent::Error {
                context: "dispatch".to_owned(),
                message: "bad command".to_owned(),
            }
        );
    }

    #[test]
    fn ipc_event_parser_accepts_subscription_event_envelopes() {
        let map_event = parse_ipc_event(
            r#"{"type":"map","handle":1,"event":{"kind":"move_end","view":{"center_lng":-123.1,"center_lat":49.2,"zoom":11.0,"bearing":0.0,"pitch":0.0}}}"#,
        )
        .unwrap();
        assert!(matches!(map_event, MapLibreEvent::Map { .. }));

        let layer_event = parse_ipc_event(
            r#"{"type":"layer","handle":1,"event":{"kind":"click","layer_id":"places","lng":-123.1,"lat":49.2,"screen_x":100.0,"screen_y":200.0,"features":[]}}"#,
        )
        .unwrap();
        assert!(matches!(layer_event, MapLibreEvent::Layer { .. }));

        let marker_drag = parse_ipc_event(
            r#"{"type":"marker_drag","marker_handle":2,"event":{"kind":"drag_end","lng":-123.1,"lat":49.2}}"#,
        )
        .unwrap();
        assert!(matches!(marker_drag, MapLibreEvent::MarkerDrag { .. }));

        let popup = parse_ipc_event(
            r#"{"type":"popup","popup_handle":4,"event":{"kind":"open","lng":-123.1,"lat":49.2}}"#,
        )
        .unwrap();
        assert!(matches!(popup, MapLibreEvent::Popup { .. }));
    }

    #[test]
    fn ipc_event_parser_returns_typed_error_for_invalid_json() {
        let error = parse_ipc_event("{broken").unwrap_err();

        assert!(
            error
                .to_string()
                .starts_with("failed to parse MapLibre event:")
        );
        assert!(std::error::Error::source(&error).is_some());
    }
}
