use crate::asset::private_index_html;
use crate::{AssetMode, MapInitOptions};
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct MapLibreViewConfig {
    pub options: MapInitOptions,
    pub asset_mode: AssetMode,
}

impl Default for MapLibreViewConfig {
    fn default() -> Self {
        Self {
            options: MapInitOptions::default(),
            asset_mode: AssetMode::default(),
        }
    }
}

impl MapLibreViewConfig {
    pub fn new(options: MapInitOptions) -> Self {
        Self {
            options,
            asset_mode: AssetMode::default(),
        }
    }

    pub fn with_asset_mode(mut self, asset_mode: AssetMode) -> Self {
        self.asset_mode = asset_mode;
        self
    }

    pub fn private_html(&self) -> String {
        private_index_html(&self.asset_mode)
    }
}

#[derive(Debug)]
pub struct MapLibreWebViewStub {
    config: MapLibreViewConfig,
    _webview: PhantomData<gpui_wry::WebView>,
}

impl MapLibreWebViewStub {
    pub fn new(config: MapLibreViewConfig) -> Self {
        Self {
            config,
            _webview: PhantomData,
        }
    }

    pub fn config(&self) -> &MapLibreViewConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_view_config_keeps_assets_private() {
        let config = MapLibreViewConfig::new(MapInitOptions::default())
            .with_asset_mode(AssetMode::VendoredPlaceholder);
        let stub = MapLibreWebViewStub::new(config);

        assert_eq!(stub.config().asset_mode, AssetMode::VendoredPlaceholder);
        assert!(stub.config().private_html().contains(r#"id="map""#));
        assert!(stub.config().private_html().contains(r#"./bridge.js"#));
    }
}
