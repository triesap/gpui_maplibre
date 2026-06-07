use crate::event::{LayerEventKind, MapEventKind, MarkerDragEventKind, PopupLifecycleEventKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapEventSubscription {
    pub kinds: Vec<MapEventKind>,
    pub throttle_ms: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerEventSubscription {
    pub layer_id: String,
    pub kinds: Vec<LayerEventKind>,
    pub throttle_ms: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkerDragEventSubscription {
    pub kinds: Vec<MarkerDragEventKind>,
    pub throttle_ms: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PopupEventSubscription {
    pub kinds: Vec<PopupLifecycleEventKind>,
}
