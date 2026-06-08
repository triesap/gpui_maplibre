#![forbid(unsafe_code)]

pub mod asset;
pub mod command;
pub mod control;
pub mod controller;
pub mod error;
pub mod event;
pub mod ids;
pub mod layer;
#[cfg(feature = "gpui-webview")]
pub mod map_view;
pub mod marker;
pub mod options;
pub mod popup;
pub mod popup_content;
pub mod runtime;
pub mod source;
pub mod subscription;
pub mod transport;
pub mod types;

mod script;

pub use asset::{
    ASSET_PROTOCOL_INDEX_PATH, ASSET_PROTOCOL_ORIGIN, ASSET_PROTOCOL_SCHEME, AssetMode,
    MapLibreAssetUrls, MapLibreAssets, ProtocolAssetResponse, protocol_asset_response,
    protocol_asset_url, protocol_index_html, protocol_webview_url,
};
pub use command::MapCommand;
pub use control::{NativeControlKind, NativeControlOptions as AddNativeControlOptions};
pub use controller::MapController;
pub use error::{MapLibreError, Result};
pub use event::{
    FeatureHit, LayerEvent, LayerEventKind, LayerFeatureHit, MapClickEvent, MapEvent, MapEventKind,
    MapLibreEvent, MapViewState, MarkerDragEvent, MarkerDragEventKind, PopupLifecycleEvent,
    PopupLifecycleEventKind, StartupTimingEvent, parse_ipc_event,
};
pub use ids::{ControlHandle, LayerId, MapHandle, MarkerHandle, PopupHandle, SourceId};
pub use layer::{LayerType, background_layer, sourced_layer};
#[cfg(feature = "gpui-webview")]
pub use map_view::{
    MapLibreView, MapLibreViewConfig, MapLibreWebViewStub, create_map_view,
    create_prewarmed_map_view,
};
pub use marker::MarkerOptions;
pub use options::{MapInitOptions, NativeControlOptions};
pub use popup::PopupOptions;
pub use popup_content::PopupContent;
pub use runtime::{
    EventRouter, EventRouterAction, EventSubscriptionRegistry, RoutedError, RuntimeCommandAction,
    RuntimeCommandQueue, ViewLifecycle, route_ipc_message,
};
pub use source::{SourceType, geojson_source, raster_source, vector_source};
pub use subscription::{
    EventSubscription, EventSubscriptionTarget, LayerEventSubscription, MapEventSubscription,
    MarkerDragEventSubscription, PopupEventSubscription,
};
#[cfg(feature = "gpui-webview")]
pub use transport::GpuiWebViewTransport;
pub use transport::{CommandTransport, FakeTransport};
pub use types::{Bounds, LngLat, MapControlAnchor};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_metadata_is_available() {
        assert_eq!(env!("CARGO_PKG_NAME"), "gpui_maplibre");
    }
}
