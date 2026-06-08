use crate::asset::{inline_webview_html, private_index_html};
use crate::runtime::{
    EventRouter, EventRouterAction, EventSubscriptionRegistry, RoutedError, ViewLifecycle,
    route_ipc_message,
};
use crate::subscription::{EventSubscription, EventSubscriptionTarget};
use crate::{
    AssetMode, CommandTransport, FakeTransport, MapController, MapHandle, MapInitOptions,
    MapLibreAssets,
};
use crate::{MapCommand, MapLibreError, MapLibreEvent, Result};
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::Rc;

type MountedIpcInbox = Rc<RefCell<VecDeque<String>>>;

/// Create a GPUI entity that owns a MapLibre WebView.
///
/// This is the happy path for applications: provide a map config, then render the returned
/// `Entity<MapLibreView>` inside the parent view. The crate owns the inline WebView HTML,
/// Wry child creation, `gpui_wry` wrapping, and IPC routing.
pub fn create_map_view<T: 'static>(
    config: MapLibreViewConfig,
    window: &mut Window,
    cx: &mut Context<T>,
) -> Result<Entity<MapLibreView>> {
    let ipc_inbox = mounted_ipc_inbox();
    let webview = build_wry_webview(&config, ipc_inbox.clone(), window)?;
    let webview = cx.new(|cx| gpui_wry::WebView::new(webview, window, cx));

    Ok(cx.new(|_| {
        let mut map_view = MapLibreView::new(config);
        map_view.set_mounted_webview(webview, ipc_inbox);
        map_view
    }))
}

fn mounted_ipc_inbox() -> MountedIpcInbox {
    Rc::new(RefCell::new(VecDeque::new()))
}

fn build_wry_webview(
    config: &MapLibreViewConfig,
    ipc_inbox: MountedIpcInbox,
    window: &mut Window,
) -> Result<wry::WebView> {
    let html = config.inline_webview_html()?;

    wry::WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |request| {
            ipc_inbox.borrow_mut().push_back(request.body().clone());
        })
        .build_as_child(window)
        .map_err(|error| MapLibreError::platform(error.to_string()))
}

/// Configuration for a mounted MapLibre view.
#[derive(Clone, Debug, PartialEq)]
pub struct MapLibreViewConfig {
    /// Initial MapLibre options dispatched when the WebView bootstraps.
    pub options: MapInitOptions,
    /// Source for MapLibre GL JS and CSS runtime assets.
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
    /// Create a config with the default CDN-backed runtime asset mode.
    pub fn new(options: MapInitOptions) -> Self {
        Self {
            options,
            asset_mode: AssetMode::default(),
        }
    }

    /// Override the MapLibre GL JS and CSS runtime assets.
    ///
    /// Use [`MapLibreAssets::vendored`] with the `vendored-maplibre` Cargo feature to run without
    /// fetching MapLibre GL JS/CSS from the network.
    pub fn with_assets(mut self, assets: MapLibreAssets) -> Self {
        self.asset_mode = assets;
        self
    }

    /// Override the MapLibre GL JS and CSS runtime assets.
    ///
    /// This compatibility helper delegates to [`Self::with_assets`].
    pub fn with_asset_mode(self, asset_mode: AssetMode) -> Self {
        self.with_assets(asset_mode)
    }

    /// Return the configured MapLibre GL JS and CSS runtime assets.
    pub fn assets(&self) -> &MapLibreAssets {
        &self.asset_mode
    }

    /// Build the private file-backed HTML used by low-level integrations.
    pub fn private_html(&self) -> String {
        private_index_html(&self.asset_mode)
    }

    /// Build inline HTML suitable for `wry::WebViewBuilder::with_html`.
    pub fn inline_webview_html(&self) -> Result<String> {
        inline_webview_html(&self.asset_mode, &self.options)
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

/// GPUI-renderable MapLibre view state.
///
/// Applications normally construct this with [`create_map_view`]. The lower-level constructors and
/// `set_webview` method remain available for custom WebView lifecycle integrations.
pub struct MapLibreView<T = FakeTransport> {
    config: MapLibreViewConfig,
    controller: MapController<T>,
    router: EventRouter,
    lifecycle: ViewLifecycle,
    subscriptions: EventSubscriptionRegistry,
    webview: Option<Entity<gpui_wry::WebView>>,
    ipc_inbox: Option<MountedIpcInbox>,
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
            lifecycle: ViewLifecycle::new(),
            subscriptions: EventSubscriptionRegistry::new(),
            webview: None,
            ipc_inbox: None,
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

    pub fn lifecycle(&self) -> &ViewLifecycle {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut ViewLifecycle {
        &mut self.lifecycle
    }

    /// Return true once the WebView bridge has emitted `dom_ready`.
    pub fn is_dom_ready(&self) -> bool {
        self.router.is_dom_ready()
    }

    /// Return true once MapLibre has emitted its ready event.
    pub fn is_map_ready(&self) -> bool {
        self.router.is_map_ready()
    }

    /// Return the current MapLibre handle after initialization.
    pub fn map_handle(&self) -> Option<MapHandle> {
        self.router.map_handle()
    }

    /// Return structured bridge errors routed from JavaScript.
    pub fn routed_errors(&self) -> &[RoutedError] {
        self.router.errors()
    }

    pub fn subscription_registry(&self) -> &EventSubscriptionRegistry {
        &self.subscriptions
    }

    pub fn subscription_registry_mut(&mut self) -> &mut EventSubscriptionRegistry {
        &mut self.subscriptions
    }

    pub fn track_subscription(
        &mut self,
        subscription: EventSubscription,
    ) -> Option<EventSubscription> {
        self.subscriptions.insert(subscription)
    }

    pub fn untrack_subscription(
        &mut self,
        target: &EventSubscriptionTarget,
    ) -> Option<EventSubscription> {
        self.subscriptions.remove(target)
    }

    pub fn is_event_subscribed(&self, event: &MapLibreEvent) -> bool {
        self.subscriptions.contains_event(event)
    }

    pub fn webview(&self) -> Option<&Entity<gpui_wry::WebView>> {
        self.webview.as_ref()
    }

    pub fn set_webview(&mut self, webview: Entity<gpui_wry::WebView>) {
        self.webview = Some(webview);
        self.ipc_inbox = None;
    }

    fn set_mounted_webview(
        &mut self,
        webview: Entity<gpui_wry::WebView>,
        ipc_inbox: MountedIpcInbox,
    ) {
        self.webview = Some(webview);
        self.ipc_inbox = Some(ipc_inbox);
    }

    pub fn take_webview(&mut self) -> Option<Entity<gpui_wry::WebView>> {
        self.ipc_inbox = None;
        self.webview.take()
    }

    pub fn last_ipc_error(&self) -> Option<&str> {
        self.last_ipc_error.as_deref()
    }

    pub fn resize_command(&self) -> Option<MapCommand> {
        self.lifecycle.resize_command()
    }

    pub fn cleanup_command(&mut self) -> Option<MapCommand> {
        let command = self.lifecycle.cleanup_command();

        if command.is_some() {
            self.controller.clear_handle();
        }

        command
    }

    pub fn handle_ipc_message(&mut self, message: &str) -> Result<EventRouterAction> {
        match route_ipc_message(&mut self.router, message) {
            Ok(action) => {
                self.sync_runtime_state(&action);
                self.last_ipc_error = None;
                Ok(action)
            }
            Err(error) => {
                self.last_ipc_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    fn drain_mounted_ipc_messages(&mut self) -> Vec<Result<EventRouterAction>> {
        let Some(ipc_inbox) = self.ipc_inbox.clone() else {
            return Vec::new();
        };
        let messages = ipc_inbox.borrow_mut().drain(..).collect::<Vec<_>>();

        messages
            .iter()
            .map(|message| self.handle_ipc_message(message))
            .collect()
    }

    fn sync_runtime_state(&mut self, action: &EventRouterAction) {
        self.lifecycle.reduce_action(action);

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

impl<T: 'static> MapLibreView<T> {
    /// Dispatch a command through the hosted WebView bridge.
    pub fn dispatch_mounted_command(
        &mut self,
        command: MapCommand,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let Some(webview) = self.webview.clone() else {
            return Err(MapLibreError::not_ready("mounted WebView is not attached"));
        };
        let script = crate::script::script_for_command(&command)?;

        cx.update_entity(&webview, |webview, _| webview.evaluate_script(&script))
            .map_err(|error| MapLibreError::platform(error.to_string()))
    }

    /// Resize the hosted WebView map if a map handle has been initialized.
    pub fn resize_mounted(&mut self, cx: &mut Context<Self>) -> Result<bool> {
        let Some(handle) = self.map_handle() else {
            return Ok(false);
        };

        self.dispatch_mounted_command(MapCommand::Resize { handle }, cx)?;
        Ok(true)
    }

    /// Destroy the hosted WebView map if a map handle has been initialized.
    pub fn cleanup_mounted(&mut self, cx: &mut Context<Self>) -> Result<bool> {
        let Some(handle) = self.map_handle() else {
            return Ok(false);
        };

        self.dispatch_mounted_command(MapCommand::Destroy { handle }, cx)?;
        self.lifecycle.clear();
        self.controller.clear_handle();
        Ok(true)
    }
}

impl<T: CommandTransport> MapLibreView<T> {
    pub fn resize(&mut self) -> Result<bool> {
        self.controller.resize_if_initialized()
    }

    pub fn cleanup(&mut self) -> Result<bool> {
        let did_cleanup = self.controller.destroy_if_initialized()?;

        if did_cleanup {
            self.lifecycle.clear();
        }

        Ok(did_cleanup)
    }

    pub fn subscribe_events(&mut self, subscription: EventSubscription) -> Result<()> {
        self.controller.subscribe_events(subscription.clone())?;
        self.subscriptions.insert(subscription);
        Ok(())
    }

    pub fn unsubscribe_events(&mut self, target: EventSubscriptionTarget) -> Result<()> {
        self.controller.unsubscribe_events(target.clone())?;
        self.subscriptions.remove(&target);
        Ok(())
    }
}

impl<T: CommandTransport + 'static> Render for MapLibreView<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.drain_mounted_ipc_messages().is_empty() {
            cx.notify();
        }

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
    use crate::{
        LayerEventSubscription, MapEventSubscription, MapHandle, MarkerDragEventSubscription,
        MarkerHandle,
    };

    #[test]
    fn map_view_config_keeps_assets_private() {
        let assets = MapLibreAssets::urls("./vendor/maplibre-gl.js", "./vendor/maplibre-gl.css");
        let config = MapLibreViewConfig::new(MapInitOptions::default()).with_assets(assets.clone());
        let stub = MapLibreWebViewStub::new(config);

        assert_eq!(stub.config().assets(), &assets);
        assert!(stub.config().private_html().contains(r#"id="map""#));
        assert!(stub.config().private_html().contains(r#"./bridge.js"#));
        assert!(
            stub.config()
                .private_html()
                .contains("./vendor/maplibre-gl.js")
        );
    }

    #[test]
    fn map_view_config_builds_inline_webview_html() {
        let config = MapLibreViewConfig::new(MapInitOptions::default().with_zoom(4.0));
        let html = config.inline_webview_html().unwrap();

        assert!(html.contains("bridge.installed_bridge.dispatch"));
        assert!(html.contains("\"zoom\":4.0"));
        assert!(!html.contains(r#"src="./bridge.js""#));
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
        assert_eq!(view.lifecycle().map_handle(), Some(MapHandle(7)));
        assert_eq!(view.controller().handle(), Some(MapHandle(7)));
        assert!(!view.is_map_ready());
        assert!(view.last_ipc_error().is_none());

        let action = view
            .handle_ipc_message(r#"{"type":"ready","handle":7}"#)
            .unwrap();

        assert_eq!(
            action,
            EventRouterAction::Ready {
                handle: MapHandle(7),
            }
        );
        assert!(view.is_map_ready());
    }

    #[test]
    fn map_view_exposes_typed_event_state() {
        let mut view = MapLibreView::new(MapLibreViewConfig::default());

        assert!(!view.is_dom_ready());
        assert!(!view.is_map_ready());
        assert_eq!(view.map_handle(), None);

        view.handle_ipc_message(r#"{"type":"dom_ready"}"#).unwrap();
        view.handle_ipc_message(r#"{"type":"ready","handle":7}"#)
            .unwrap();
        view.handle_ipc_message(r#"{"type":"error","context":"dispatch","message":"bad command"}"#)
            .unwrap();

        assert!(view.is_dom_ready());
        assert!(view.is_map_ready());
        assert_eq!(view.map_handle(), Some(MapHandle(7)));
        assert_eq!(view.routed_errors().len(), 1);
        assert_eq!(view.routed_errors()[0].context, "dispatch");
    }

    #[test]
    fn mounted_ipc_inbox_routes_messages_into_view_state() {
        let ipc_inbox = mounted_ipc_inbox();
        ipc_inbox
            .borrow_mut()
            .push_back(r#"{"type":"dom_ready"}"#.to_owned());
        ipc_inbox
            .borrow_mut()
            .push_back(r#"{"type":"initialized","handle":7}"#.to_owned());
        ipc_inbox
            .borrow_mut()
            .push_back(r#"{"type":"ready","handle":7}"#.to_owned());
        let mut view = MapLibreView::new(MapLibreViewConfig::default());
        view.ipc_inbox = Some(ipc_inbox);

        let actions = view.drain_mounted_ipc_messages();

        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].as_ref().unwrap(), &EventRouterAction::DomReady);
        assert_eq!(
            actions[1].as_ref().unwrap(),
            &EventRouterAction::Initialized {
                handle: MapHandle(7),
            }
        );
        assert_eq!(
            actions[2].as_ref().unwrap(),
            &EventRouterAction::Ready {
                handle: MapHandle(7),
            }
        );
        assert!(view.router().is_dom_ready());
        assert_eq!(view.router().initialized_handle(), Some(MapHandle(7)));
        assert_eq!(view.router().ready_handle(), Some(MapHandle(7)));
        assert!(view.is_map_ready());
        assert_eq!(view.lifecycle().map_handle(), Some(MapHandle(7)));
        assert_eq!(view.controller().handle(), Some(MapHandle(7)));
    }

    #[test]
    fn lifecycle_cleanup_map_view_exposes_resize_and_cleanup_commands() {
        let mut view = MapLibreView::new(MapLibreViewConfig::default());
        view.handle_ipc_message(r#"{"type":"ready","handle":7}"#)
            .unwrap();

        assert_eq!(
            view.resize_command(),
            Some(MapCommand::Resize {
                handle: MapHandle(7),
            })
        );
        assert_eq!(
            view.cleanup_command(),
            Some(MapCommand::Destroy {
                handle: MapHandle(7),
            })
        );
        assert_eq!(view.cleanup_command(), None);
        assert_eq!(view.controller().handle(), None);
    }

    #[test]
    fn lifecycle_cleanup_map_view_can_dispatch_through_controller() {
        let mut view = MapLibreView::new(MapLibreViewConfig::default());
        view.handle_ipc_message(r#"{"type":"ready","handle":7}"#)
            .unwrap();

        assert!(view.resize().unwrap());
        assert!(view.cleanup().unwrap());
        assert!(!view.cleanup().unwrap());

        assert_eq!(
            view.controller().transport().commands(),
            &[
                MapCommand::Resize {
                    handle: MapHandle(7),
                },
                MapCommand::Destroy {
                    handle: MapHandle(7),
                },
            ]
        );
    }

    #[test]
    fn subscriptions_map_view_tracks_generic_event_api_after_dispatch() {
        let mut view = MapLibreView::new(MapLibreViewConfig::default());
        view.handle_ipc_message(r#"{"type":"ready","handle":7}"#)
            .unwrap();

        view.subscribe_events(EventSubscription::map(MapEventSubscription::default()))
            .unwrap();
        view.subscribe_events(EventSubscription::layer(LayerEventSubscription::clicks(
            "places",
        )))
        .unwrap();
        view.subscribe_events(EventSubscription::marker_drag(
            MarkerHandle(10),
            MarkerDragEventSubscription::default(),
        ))
        .unwrap();
        view.unsubscribe_events(EventSubscriptionTarget::layer("places"))
            .unwrap();

        assert!(
            view.subscription_registry()
                .contains(&EventSubscriptionTarget::Map)
        );
        assert!(
            !view
                .subscription_registry()
                .contains(&EventSubscriptionTarget::layer("places"))
        );
        assert!(
            view.subscription_registry()
                .contains(&EventSubscriptionTarget::marker_drag(MarkerHandle(10)))
        );
        assert_eq!(
            view.controller().transport().commands(),
            &[
                MapCommand::SubscribeMapEvents {
                    handle: MapHandle(7),
                    subscription: MapEventSubscription::default(),
                },
                MapCommand::SubscribeLayerEvents {
                    handle: MapHandle(7),
                    subscription: LayerEventSubscription::clicks("places"),
                },
                MapCommand::SubscribeMarkerDragEvents {
                    marker_handle: MarkerHandle(10),
                    subscription: MarkerDragEventSubscription::default(),
                },
                MapCommand::UnsubscribeLayerEvents {
                    handle: MapHandle(7),
                    layer_id: "places".to_owned(),
                },
            ]
        );
    }
}
