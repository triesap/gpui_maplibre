#![forbid(unsafe_code)]

pub mod command;
pub mod control;
pub mod error;
pub mod event;
pub mod ids;
pub mod marker;
pub mod options;
pub mod popup;
pub mod subscription;
pub mod types;

pub use command::MapCommand;
pub use control::{NativeControlKind, NativeControlOptions as AddNativeControlOptions};
pub use error::{MapLibreError, Result};
pub use event::{
    FeatureHit, LayerEvent, LayerEventKind, LayerFeatureHit, MapClickEvent, MapEvent, MapEventKind,
    MapViewState, MarkerDragEvent, MarkerDragEventKind, PopupLifecycleEvent,
    PopupLifecycleEventKind,
};
pub use ids::{ControlHandle, LayerId, MapHandle, MarkerHandle, PopupHandle, SourceId};
pub use marker::MarkerOptions;
pub use options::{MapInitOptions, NativeControlOptions};
pub use popup::{PopupContent, PopupOptions};
pub use subscription::{
    LayerEventSubscription, MapEventSubscription, MarkerDragEventSubscription,
    PopupEventSubscription,
};
pub use types::{Bounds, LngLat, MapControlAnchor};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_metadata_is_available() {
        assert_eq!(env!("CARGO_PKG_NAME"), "gpui_maplibre");
    }
}
