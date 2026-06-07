use crate::asset::private_index_html;
use crate::runtime::{EventRouter, EventRouterAction, route_ipc_message};
use crate::{AssetMode, MapInitOptions};
use crate::{MapLibreError, Result};
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
    router: EventRouter,
    last_ipc_error: Option<String>,
    _webview: PhantomData<gpui_wry::WebView>,
}

impl MapLibreWebViewStub {
    pub fn new(config: MapLibreViewConfig) -> Self {
        Self {
            config,
            router: EventRouter::new(),
            last_ipc_error: None,
            _webview: PhantomData,
        }
    }

    pub fn config(&self) -> &MapLibreViewConfig {
        &self.config
    }

    pub fn router(&self) -> &EventRouter {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut EventRouter {
        &mut self.router
    }

    pub fn last_ipc_error(&self) -> Option<&str> {
        self.last_ipc_error.as_deref()
    }

    pub fn handle_ipc_message(&mut self, message: &str) -> Result<EventRouterAction> {
        match route_ipc_message(&mut self.router, message) {
            Ok(action) => {
                self.last_ipc_error = None;
                Ok(action)
            }
            Err(error) => {
                self.last_ipc_error = Some(error.to_string());
                Err(error)
            }
        }
    }
}

pub fn handle_webview_ipc(
    router: &mut EventRouter,
    message: &str,
) -> std::result::Result<EventRouterAction, MapLibreError> {
    route_ipc_message(router, message)
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

    #[test]
    fn map_view_stub_routes_ipc_messages() {
        let mut stub = MapLibreWebViewStub::new(MapLibreViewConfig::default());

        let action = stub.handle_ipc_message(r#"{"type":"dom_ready"}"#).unwrap();

        assert_eq!(action, EventRouterAction::DomReady);
        assert!(stub.router().is_dom_ready());
        assert!(stub.last_ipc_error().is_none());

        let error = stub.handle_ipc_message("{broken").unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("failed to parse MapLibre event:")
        );
        assert!(stub.last_ipc_error().is_some());
    }
}
