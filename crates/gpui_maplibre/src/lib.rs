#![forbid(unsafe_code)]

mod error;

pub mod ids;
pub mod options;
pub mod types;

pub use ids::{ControlHandle, LayerId, MapHandle, MarkerHandle, PopupHandle, SourceId};
pub use options::{MapInitOptions, NativeControlOptions};
pub use types::{Bounds, LngLat, MapControlAnchor};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_metadata_is_available() {
        assert_eq!(env!("CARGO_PKG_NAME"), "gpui_maplibre");
    }
}
