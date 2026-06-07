use crate::event::{LayerEventKind, MapEventKind, MarkerDragEventKind, PopupLifecycleEventKind};
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
