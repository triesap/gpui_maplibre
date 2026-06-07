use crate::asset::private_index_html;
use crate::runtime::{EventRouter, EventRouterAction, route_ipc_message};
use crate::{AssetMode, CommandTransport, FakeTransport, MapController, MapInitOptions};
use crate::{MapLibreError, Result};
use gpui::{Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window, div};
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

pub struct MapLibreView<T = FakeTransport> {
    config: MapLibreViewConfig,
    controller: MapController<T>,
    router: EventRouter,
    webview: Option<Entity<gpui_wry::WebView>>,
    last_ipc_error: Option<String>,
}

impl MapLibreView<FakeTransport> {
    pub fn new(config: MapLibreViewConfig) -> Self {
        Self::with_transport(config, FakeTransport::new())
    }
}

impl<T> MapLibreView<T> {
    pub fn with_transport(config: MapLibreViewConfig, transport: T) -> Self {
        Self::with_controller(config, MapController::new(transport))
    }

    pub fn with_controller(config: MapLibreViewConfig, controller: MapController<T>) -> Self {
        Self {
            config,
            controller,
            router: EventRouter::new(),
            webview: None,
            last_ipc_error: None,
        }
    }

    pub fn config(&self) -> &MapLibreViewConfig {
        &self.config
    }

    pub fn controller(&self) -> &MapController<T> {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut MapController<T> {
        &mut self.controller
    }

    pub fn router(&self) -> &EventRouter {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut EventRouter {
        &mut self.router
    }

    pub fn webview(&self) -> Option<&Entity<gpui_wry::WebView>> {
        self.webview.as_ref()
    }

    pub fn set_webview(&mut self, webview: Entity<gpui_wry::WebView>) {
        self.webview = Some(webview);
    }

    pub fn take_webview(&mut self) -> Option<Entity<gpui_wry::WebView>> {
        self.webview.take()
    }

    pub fn last_ipc_error(&self) -> Option<&str> {
        self.last_ipc_error.as_deref()
    }

    pub fn handle_ipc_message(&mut self, message: &str) -> Result<EventRouterAction> {
        match route_ipc_message(&mut self.router, message) {
            Ok(action) => {
                self.sync_controller_handle(&action);
                self.last_ipc_error = None;
                Ok(action)
            }
            Err(error) => {
                self.last_ipc_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    fn sync_controller_handle(&mut self, action: &EventRouterAction) {
        match action {
            EventRouterAction::Initialized { handle } | EventRouterAction::Ready { handle } => {
                self.controller.set_handle(*handle);
            }
            EventRouterAction::DomReady
            | EventRouterAction::NativeControlCreated { .. }
            | EventRouterAction::MarkerCreated { .. }
            | EventRouterAction::PopupCreated { .. }
            | EventRouterAction::Emit(_)
            | EventRouterAction::Error(_) => {}
        }
    }
}

impl<T: CommandTransport + 'static> Render for MapLibreView<T> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = div().size_full();

        if let Some(webview) = self.webview.as_ref() {
            container = container.child(webview.clone());
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MapHandle;

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

    #[test]
    fn map_view_wrapper_keeps_config_controller_and_router_state() {
        let mut view = MapLibreView::new(MapLibreViewConfig::default());

        assert!(view.config().private_html().contains(r#"id="map""#));
        assert_eq!(view.controller().handle(), None);
        assert!(view.webview().is_none());

        let action = view
            .handle_ipc_message(r#"{"type":"initialized","handle":7}"#)
            .unwrap();

        assert_eq!(
            action,
            EventRouterAction::Initialized {
                handle: MapHandle(7),
            }
        );
        assert_eq!(view.router().map_handle(), Some(MapHandle(7)));
        assert_eq!(view.controller().handle(), Some(MapHandle(7)));
        assert!(view.last_ipc_error().is_none());
    }
}
