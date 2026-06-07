#![forbid(unsafe_code)]

mod error;

pub mod ids;

pub use ids::{ControlHandle, LayerId, MapHandle, MarkerHandle, PopupHandle, SourceId};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_metadata_is_available() {
        assert_eq!(env!("CARGO_PKG_NAME"), "gpui_maplibre");
    }
}
