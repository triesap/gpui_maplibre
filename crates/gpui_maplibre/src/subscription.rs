use crate::event::{LayerEventKind, MapEventKind, MarkerDragEventKind, PopupLifecycleEventKind};
use crate::ids::{LayerId, MarkerHandle, PopupHandle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapEventSubscription {
    pub kinds: Vec<MapEventKind>,
    pub throttle_ms: Option<u32>,
}

impl MapEventSubscription {
    pub fn new(kinds: impl IntoIterator<Item = MapEventKind>) -> Self {
        Self {
            kinds: kinds.into_iter().collect(),
            throttle_ms: None,
        }
    }

    pub fn with_throttle_ms(mut self, throttle_ms: u32) -> Self {
        self.throttle_ms = Some(throttle_ms);
        self
    }
}

impl Default for MapEventSubscription {
    fn default() -> Self {
        Self::new([
            MapEventKind::MoveEnd,
            MapEventKind::ZoomEnd,
            MapEventKind::Idle,
            MapEventKind::Resize,
            MapEventKind::Error,
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerEventSubscription {
    pub layer_id: String,
    pub kinds: Vec<LayerEventKind>,
    pub throttle_ms: Option<u32>,
}

impl LayerEventSubscription {
    pub fn new(
        layer_id: impl Into<String>,
        kinds: impl IntoIterator<Item = LayerEventKind>,
    ) -> Self {
        Self {
            layer_id: layer_id.into(),
            kinds: kinds.into_iter().collect(),
            throttle_ms: None,
        }
    }

    pub fn clicks(layer_id: impl Into<String>) -> Self {
        Self::new(layer_id, [LayerEventKind::Click])
    }

    pub fn with_throttle_ms(mut self, throttle_ms: u32) -> Self {
        self.throttle_ms = Some(throttle_ms);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkerDragEventSubscription {
    pub kinds: Vec<MarkerDragEventKind>,
    pub throttle_ms: Option<u32>,
}

impl MarkerDragEventSubscription {
    pub fn new(kinds: impl IntoIterator<Item = MarkerDragEventKind>) -> Self {
        Self {
            kinds: kinds.into_iter().collect(),
            throttle_ms: None,
        }
    }

    pub fn with_throttle_ms(mut self, throttle_ms: u32) -> Self {
        self.throttle_ms = Some(throttle_ms);
        self
    }
}

impl Default for MarkerDragEventSubscription {
    fn default() -> Self {
        Self::new([
            MarkerDragEventKind::DragStart,
            MarkerDragEventKind::Drag,
            MarkerDragEventKind::DragEnd,
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PopupEventSubscription {
    pub kinds: Vec<PopupLifecycleEventKind>,
}

impl PopupEventSubscription {
    pub fn new(kinds: impl IntoIterator<Item = PopupLifecycleEventKind>) -> Self {
        Self {
            kinds: kinds.into_iter().collect(),
        }
    }
}

impl Default for PopupEventSubscription {
    fn default() -> Self {
        Self::new([
            PopupLifecycleEventKind::Open,
            PopupLifecycleEventKind::Close,
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum EventSubscription {
    Map {
        subscription: MapEventSubscription,
    },
    Layer {
        subscription: LayerEventSubscription,
    },
    MarkerDrag {
        marker_handle: MarkerHandle,
        subscription: MarkerDragEventSubscription,
    },
    Popup {
        popup_handle: PopupHandle,
        subscription: PopupEventSubscription,
    },
}

impl EventSubscription {
    pub fn map(subscription: MapEventSubscription) -> Self {
        Self::Map { subscription }
    }

    pub fn layer(subscription: LayerEventSubscription) -> Self {
        Self::Layer { subscription }
    }

    pub fn marker_drag(
        marker_handle: MarkerHandle,
        subscription: MarkerDragEventSubscription,
    ) -> Self {
        Self::MarkerDrag {
            marker_handle,
            subscription,
        }
    }

    pub fn popup(popup_handle: PopupHandle, subscription: PopupEventSubscription) -> Self {
        Self::Popup {
            popup_handle,
            subscription,
        }
    }

    pub fn target(&self) -> EventSubscriptionTarget {
        match self {
            Self::Map { .. } => EventSubscriptionTarget::Map,
            Self::Layer { subscription } => EventSubscriptionTarget::Layer {
                layer_id: subscription.layer_id.clone(),
            },
            Self::MarkerDrag { marker_handle, .. } => EventSubscriptionTarget::MarkerDrag {
                marker_handle: *marker_handle,
            },
            Self::Popup { popup_handle, .. } => EventSubscriptionTarget::Popup {
                popup_handle: *popup_handle,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum EventSubscriptionTarget {
    Map,
    Layer { layer_id: String },
    MarkerDrag { marker_handle: MarkerHandle },
    Popup { popup_handle: PopupHandle },
}

impl EventSubscriptionTarget {
    pub fn map() -> Self {
        Self::Map
    }

    pub fn layer(layer_id: impl Into<LayerId>) -> Self {
        Self::Layer {
            layer_id: layer_id.into().to_string(),
        }
    }

    pub fn marker_drag(marker_handle: MarkerHandle) -> Self {
        Self::MarkerDrag { marker_handle }
    }

    pub fn popup(popup_handle: PopupHandle) -> Self {
        Self::Popup { popup_handle }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn subscriptions_build_defaults_and_custom_kinds() {
        assert_eq!(
            MapEventSubscription::default(),
            MapEventSubscription {
                kinds: vec![
                    MapEventKind::MoveEnd,
                    MapEventKind::ZoomEnd,
                    MapEventKind::Idle,
                    MapEventKind::Resize,
                    MapEventKind::Error,
                ],
                throttle_ms: None,
            }
        );

        assert_eq!(
            LayerEventSubscription::clicks("places").with_throttle_ms(100),
            LayerEventSubscription {
                layer_id: "places".to_owned(),
                kinds: vec![LayerEventKind::Click],
                throttle_ms: Some(100),
            }
        );

        assert_eq!(
            MarkerDragEventSubscription::default().with_throttle_ms(16),
            MarkerDragEventSubscription {
                kinds: vec![
                    MarkerDragEventKind::DragStart,
                    MarkerDragEventKind::Drag,
                    MarkerDragEventKind::DragEnd,
                ],
                throttle_ms: Some(16),
            }
        );

        assert_eq!(
            PopupEventSubscription::default(),
            PopupEventSubscription {
                kinds: vec![
                    PopupLifecycleEventKind::Open,
                    PopupLifecycleEventKind::Close
                ],
            }
        );
    }

    #[test]
    fn subscriptions_build_generic_targets() {
        let map = EventSubscription::map(MapEventSubscription::default().with_throttle_ms(250));
        let layer = EventSubscription::layer(LayerEventSubscription::clicks("places"));
        let marker = EventSubscription::marker_drag(
            MarkerHandle(10),
            MarkerDragEventSubscription::default().with_throttle_ms(16),
        );
        let popup = EventSubscription::popup(PopupHandle(11), PopupEventSubscription::default());

        assert_eq!(map.target(), EventSubscriptionTarget::Map);
        assert_eq!(
            layer.target(),
            EventSubscriptionTarget::Layer {
                layer_id: "places".to_owned(),
            }
        );
        assert_eq!(
            marker.target(),
            EventSubscriptionTarget::MarkerDrag {
                marker_handle: MarkerHandle(10),
            }
        );
        assert_eq!(
            popup.target(),
            EventSubscriptionTarget::Popup {
                popup_handle: PopupHandle(11),
            }
        );
    }

    #[test]
    fn subscriptions_serialize_generic_envelope() {
        assert_eq!(
            serde_json::to_value(EventSubscriptionTarget::layer("places")).unwrap(),
            json!({
                "scope": "layer",
                "layer_id": "places"
            })
        );

        assert_eq!(
            serde_json::to_value(EventSubscription::popup(
                PopupHandle(11),
                PopupEventSubscription::default()
            ))
            .unwrap(),
            json!({
                "scope": "popup",
                "popup_handle": 11,
                "subscription": {
                    "kinds": ["open", "close"]
                }
            })
        );
    }
}
