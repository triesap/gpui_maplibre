use crate::control::NativeControlKind;
use crate::ids::{ControlHandle, LayerId, MapHandle, MarkerHandle, PopupHandle, SourceId};
use crate::marker::MarkerOptions;
use crate::options::MapInitOptions;
use crate::popup::PopupOptions;
use crate::subscription::{
    EventSubscription, EventSubscriptionTarget, LayerEventSubscription, MapEventSubscription,
    MarkerDragEventSubscription, PopupEventSubscription,
};
use crate::transport::CommandTransport;
use crate::types::{Bounds, LngLat, MapControlAnchor};
use crate::{MapCommand, MapLibreError, Result};

#[derive(Clone, Debug)]
pub struct MapController<T> {
    transport: T,
    handle: Option<MapHandle>,
    next_request_id: u64,
}

impl<T> MapController<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            handle: None,
            next_request_id: 1,
        }
    }

    pub fn with_handle(transport: T, handle: MapHandle) -> Self {
        Self {
            transport,
            handle: Some(handle),
            next_request_id: 1,
        }
    }

    pub fn handle(&self) -> Option<MapHandle> {
        self.handle
    }

    pub fn set_handle(&mut self, handle: MapHandle) {
        self.handle = Some(handle);
    }

    pub fn clear_handle(&mut self) {
        self.handle = None;
    }

    pub fn next_request_id(&self) -> u64 {
        self.next_request_id
    }

    pub fn set_next_request_id(&mut self, next_request_id: u64) {
        self.next_request_id = next_request_id;
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    pub fn into_transport(self) -> T {
        self.transport
    }
}

impl<T: CommandTransport> MapController<T> {
    pub fn init(&mut self, options: MapInitOptions) -> Result<()> {
        self.send(MapCommand::Init { options })
    }

    pub fn destroy(&mut self) -> Result<()> {
        let handle = self.require_handle("destroy requires an initialized map handle")?;

        self.send(MapCommand::Destroy { handle })?;
        self.handle = None;
        Ok(())
    }

    pub fn destroy_if_initialized(&mut self) -> Result<bool> {
        let Some(handle) = self.handle else {
            return Ok(false);
        };

        self.send(MapCommand::Destroy { handle })?;
        self.handle = None;
        Ok(true)
    }

    pub fn resize(&mut self) -> Result<()> {
        let handle = self.require_handle("resize requires an initialized map handle")?;

        self.send(MapCommand::Resize { handle })
    }

    pub fn resize_if_initialized(&mut self) -> Result<bool> {
        let Some(handle) = self.handle else {
            return Ok(false);
        };

        self.send(MapCommand::Resize { handle })?;
        Ok(true)
    }

    pub fn set_style(&mut self, style_url: impl Into<String>) -> Result<()> {
        let handle = self.require_handle("set_style requires an initialized map handle")?;

        self.send(MapCommand::SetStyle {
            handle,
            style_url: style_url.into(),
        })
    }

    pub fn fly_to(
        &mut self,
        center: LngLat,
        zoom: Option<f64>,
        duration_ms: Option<u32>,
    ) -> Result<()> {
        let handle = self.require_handle("fly_to requires an initialized map handle")?;

        self.send(MapCommand::FlyTo {
            handle,
            lng: center.lng,
            lat: center.lat,
            zoom,
            duration_ms,
        })
    }

    pub fn jump_to(
        &mut self,
        center: LngLat,
        zoom: Option<f64>,
        bearing: Option<f64>,
        pitch: Option<f64>,
    ) -> Result<()> {
        let handle = self.require_handle("jump_to requires an initialized map handle")?;

        self.send(MapCommand::JumpTo {
            handle,
            lng: center.lng,
            lat: center.lat,
            zoom,
            bearing,
            pitch,
        })
    }

    pub fn ease_to(
        &mut self,
        center: LngLat,
        zoom: Option<f64>,
        bearing: Option<f64>,
        pitch: Option<f64>,
        duration_ms: Option<u32>,
    ) -> Result<()> {
        let handle = self.require_handle("ease_to requires an initialized map handle")?;

        self.send(MapCommand::EaseTo {
            handle,
            lng: center.lng,
            lat: center.lat,
            zoom,
            bearing,
            pitch,
            duration_ms,
        })
    }

    pub fn fit_bounds(
        &mut self,
        bounds: Bounds,
        padding: Option<f64>,
        duration_ms: Option<u32>,
        max_zoom: Option<f64>,
    ) -> Result<()> {
        let handle = self.require_handle("fit_bounds requires an initialized map handle")?;

        self.send(MapCommand::FitBounds {
            handle,
            west: bounds.west,
            south: bounds.south,
            east: bounds.east,
            north: bounds.north,
            padding,
            duration_ms,
            max_zoom,
        })
    }

    pub fn add_source(
        &mut self,
        source_id: impl Into<SourceId>,
        source_spec: serde_json::Value,
    ) -> Result<()> {
        let handle = self.require_handle("add_source requires an initialized map handle")?;

        self.send(MapCommand::AddSource {
            handle,
            source_id: source_id.into().to_string(),
            source_spec,
        })
    }

    pub fn add_geojson_source(
        &mut self,
        source_id: impl Into<SourceId>,
        geojson: serde_json::Value,
        promote_id: Option<String>,
    ) -> Result<()> {
        let handle =
            self.require_handle("add_geojson_source requires an initialized map handle")?;

        self.send(MapCommand::AddGeoJsonSource {
            handle,
            source_id: source_id.into().to_string(),
            geojson,
            promote_id,
        })
    }

    pub fn update_geojson_source(
        &mut self,
        source_id: impl Into<SourceId>,
        geojson: serde_json::Value,
    ) -> Result<()> {
        let handle =
            self.require_handle("update_geojson_source requires an initialized map handle")?;

        self.send(MapCommand::UpdateGeoJsonSource {
            handle,
            source_id: source_id.into().to_string(),
            geojson,
        })
    }

    pub fn remove_source(&mut self, source_id: impl Into<SourceId>) -> Result<()> {
        let handle = self.require_handle("remove_source requires an initialized map handle")?;

        self.send(MapCommand::RemoveSource {
            handle,
            source_id: source_id.into().to_string(),
        })
    }

    pub fn add_layer(
        &mut self,
        layer_id: impl Into<LayerId>,
        layer_spec: serde_json::Value,
        before_id: Option<String>,
    ) -> Result<()> {
        let handle = self.require_handle("add_layer requires an initialized map handle")?;

        self.send(MapCommand::AddLayer {
            handle,
            layer_id: layer_id.into().to_string(),
            layer_spec,
            before_id,
        })
    }

    pub fn remove_layer(&mut self, layer_id: impl Into<LayerId>) -> Result<()> {
        let handle = self.require_handle("remove_layer requires an initialized map handle")?;

        self.send(MapCommand::RemoveLayer {
            handle,
            layer_id: layer_id.into().to_string(),
        })
    }

    pub fn set_layout_property(
        &mut self,
        layer_id: impl Into<LayerId>,
        property_name: impl Into<String>,
        value: serde_json::Value,
    ) -> Result<()> {
        let handle =
            self.require_handle("set_layout_property requires an initialized map handle")?;

        self.send(MapCommand::SetLayoutProperty {
            handle,
            layer_id: layer_id.into().to_string(),
            property_name: property_name.into(),
            value,
        })
    }

    pub fn set_paint_property(
        &mut self,
        layer_id: impl Into<LayerId>,
        property_name: impl Into<String>,
        value: serde_json::Value,
    ) -> Result<()> {
        let handle =
            self.require_handle("set_paint_property requires an initialized map handle")?;

        self.send(MapCommand::SetPaintProperty {
            handle,
            layer_id: layer_id.into().to_string(),
            property_name: property_name.into(),
            value,
        })
    }

    pub fn set_filter(
        &mut self,
        layer_id: impl Into<LayerId>,
        filter: Option<serde_json::Value>,
    ) -> Result<()> {
        let handle = self.require_handle("set_filter requires an initialized map handle")?;

        self.send(MapCommand::SetFilter {
            handle,
            layer_id: layer_id.into().to_string(),
            filter,
        })
    }

    pub fn set_layer_zoom_range(
        &mut self,
        layer_id: impl Into<LayerId>,
        min_zoom: Option<f64>,
        max_zoom: Option<f64>,
    ) -> Result<()> {
        let handle =
            self.require_handle("set_layer_zoom_range requires an initialized map handle")?;

        self.send(MapCommand::SetLayerZoomRange {
            handle,
            layer_id: layer_id.into().to_string(),
            min_zoom,
            max_zoom,
        })
    }

    pub fn set_feature_state(
        &mut self,
        source_id: impl Into<SourceId>,
        source_layer: Option<String>,
        feature_id: serde_json::Value,
        state: serde_json::Value,
    ) -> Result<()> {
        let handle = self.require_handle("set_feature_state requires an initialized map handle")?;

        self.send(MapCommand::SetFeatureState {
            handle,
            source_id: source_id.into().to_string(),
            source_layer,
            feature_id,
            state,
        })
    }

    pub fn set_terrain(&mut self, terrain: Option<serde_json::Value>) -> Result<()> {
        let handle = self.require_handle("set_terrain requires an initialized map handle")?;

        self.send(MapCommand::SetTerrain { handle, terrain })
    }

    pub fn set_fog(&mut self, fog: Option<serde_json::Value>) -> Result<()> {
        let handle = self.require_handle("set_fog requires an initialized map handle")?;

        self.send(MapCommand::SetFog { handle, fog })
    }

    pub fn set_light(&mut self, light: Option<serde_json::Value>) -> Result<()> {
        let handle = self.require_handle("set_light requires an initialized map handle")?;

        self.send(MapCommand::SetLight { handle, light })
    }

    pub fn add_native_control(
        &mut self,
        kind: NativeControlKind,
        anchor: Option<MapControlAnchor>,
        options: Option<serde_json::Value>,
    ) -> Result<u64> {
        let handle =
            self.require_handle("add_native_control requires an initialized map handle")?;

        self.send_request_command(|request_id| MapCommand::AddNativeControl {
            request_id,
            handle,
            kind,
            anchor,
            options,
        })
    }

    pub fn remove_native_control(&mut self, control_handle: ControlHandle) -> Result<()> {
        self.send(MapCommand::RemoveNativeControl { control_handle })
    }

    pub fn create_marker(&mut self, options: MarkerOptions) -> Result<u64> {
        let handle = self.require_handle("create_marker requires an initialized map handle")?;

        self.send_request_command(|request_id| MapCommand::CreateMarker {
            request_id,
            handle,
            options,
        })
    }

    pub fn update_marker(
        &mut self,
        marker_handle: MarkerHandle,
        options: MarkerOptions,
    ) -> Result<()> {
        self.send(MapCommand::UpdateMarker {
            marker_handle,
            options,
        })
    }

    pub fn remove_marker(&mut self, marker_handle: MarkerHandle) -> Result<()> {
        self.send(MapCommand::RemoveMarker { marker_handle })
    }

    pub fn create_popup(&mut self, options: PopupOptions) -> Result<u64> {
        let handle = self.require_handle("create_popup requires an initialized map handle")?;

        self.send_request_command(|request_id| MapCommand::CreatePopup {
            request_id,
            handle,
            options,
        })
    }

    pub fn update_popup(&mut self, popup_handle: PopupHandle, options: PopupOptions) -> Result<()> {
        self.send(MapCommand::UpdatePopup {
            popup_handle,
            options,
        })
    }

    pub fn remove_popup(&mut self, popup_handle: PopupHandle) -> Result<()> {
        self.send(MapCommand::RemovePopup { popup_handle })
    }

    pub fn subscribe_map_events(&mut self, subscription: MapEventSubscription) -> Result<()> {
        let handle =
            self.require_handle("subscribe_map_events requires an initialized map handle")?;

        self.send(MapCommand::SubscribeMapEvents {
            handle,
            subscription,
        })
    }

    pub fn unsubscribe_map_events(&mut self) -> Result<()> {
        let handle =
            self.require_handle("unsubscribe_map_events requires an initialized map handle")?;

        self.send(MapCommand::UnsubscribeMapEvents { handle })
    }

    pub fn subscribe_layer_events(&mut self, subscription: LayerEventSubscription) -> Result<()> {
        let handle =
            self.require_handle("subscribe_layer_events requires an initialized map handle")?;

        self.send(MapCommand::SubscribeLayerEvents {
            handle,
            subscription,
        })
    }

    pub fn unsubscribe_layer_events(&mut self, layer_id: impl Into<LayerId>) -> Result<()> {
        let handle =
            self.require_handle("unsubscribe_layer_events requires an initialized map handle")?;

        self.send(MapCommand::UnsubscribeLayerEvents {
            handle,
            layer_id: layer_id.into().to_string(),
        })
    }

    pub fn subscribe_marker_drag_events(
        &mut self,
        marker_handle: MarkerHandle,
        subscription: MarkerDragEventSubscription,
    ) -> Result<()> {
        self.send(MapCommand::SubscribeMarkerDragEvents {
            marker_handle,
            subscription,
        })
    }

    pub fn unsubscribe_marker_drag_events(&mut self, marker_handle: MarkerHandle) -> Result<()> {
        self.send(MapCommand::UnsubscribeMarkerDragEvents { marker_handle })
    }

    pub fn subscribe_popup_events(
        &mut self,
        popup_handle: PopupHandle,
        subscription: PopupEventSubscription,
    ) -> Result<()> {
        self.send(MapCommand::SubscribePopupEvents {
            popup_handle,
            subscription,
        })
    }

    pub fn unsubscribe_popup_events(&mut self, popup_handle: PopupHandle) -> Result<()> {
        self.send(MapCommand::UnsubscribePopupEvents { popup_handle })
    }

    pub fn subscribe_events(&mut self, subscription: EventSubscription) -> Result<()> {
        match subscription {
            EventSubscription::Map { subscription } => self.subscribe_map_events(subscription),
            EventSubscription::Layer { subscription } => self.subscribe_layer_events(subscription),
            EventSubscription::MarkerDrag {
                marker_handle,
                subscription,
            } => self.subscribe_marker_drag_events(marker_handle, subscription),
            EventSubscription::Popup {
                popup_handle,
                subscription,
            } => self.subscribe_popup_events(popup_handle, subscription),
        }
    }

    pub fn unsubscribe_events(&mut self, target: EventSubscriptionTarget) -> Result<()> {
        match target {
            EventSubscriptionTarget::Map => self.unsubscribe_map_events(),
            EventSubscriptionTarget::Layer { layer_id } => self.unsubscribe_layer_events(layer_id),
            EventSubscriptionTarget::MarkerDrag { marker_handle } => {
                self.unsubscribe_marker_drag_events(marker_handle)
            }
            EventSubscriptionTarget::Popup { popup_handle } => {
                self.unsubscribe_popup_events(popup_handle)
            }
        }
    }

    fn send(&mut self, command: MapCommand) -> Result<()> {
        self.transport.send_command(command)
    }

    fn send_request_command(
        &mut self,
        make_command: impl FnOnce(u64) -> MapCommand,
    ) -> Result<u64> {
        let request_id = self.next_request_id;

        self.send(make_command(request_id))?;
        self.next_request_id += 1;
        Ok(request_id)
    }

    fn require_handle(&self, context: &'static str) -> Result<MapHandle> {
        self.handle
            .ok_or_else(|| MapLibreError::missing_handle(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FakeTransport, LayerType, PopupContent, SourceType};
    use serde_json::json;

    fn marker_options(lng: f64, lat: f64) -> MarkerOptions {
        MarkerOptions {
            lng,
            lat,
            draggable: false,
            anchor: None,
            offset_x: None,
            offset_y: None,
            rotation: None,
        }
    }

    fn popup_options(lng: f64, lat: f64, content: impl Into<String>) -> PopupOptions {
        PopupOptions {
            lng,
            lat,
            content: PopupContent::Text(content.into()),
            close_button: Some(true),
            close_on_click: Some(false),
            anchor: None,
            offset_x: None,
            offset_y: None,
            max_width: None,
        }
    }

    #[test]
    fn controller_lifecycle_sends_init_resize_set_style_and_destroy() {
        let mut controller = MapController::new(FakeTransport::new());

        controller
            .init(MapInitOptions::default().with_style_url("maplibre://styles/basic"))
            .unwrap();
        controller.set_handle(MapHandle(1));
        controller.resize().unwrap();
        controller.set_style("maplibre://styles/satellite").unwrap();
        controller.destroy().unwrap();

        assert_eq!(controller.handle(), None);
        assert_eq!(
            controller.transport().commands(),
            &[
                MapCommand::Init {
                    options: MapInitOptions::default().with_style_url("maplibre://styles/basic"),
                },
                MapCommand::Resize {
                    handle: MapHandle(1),
                },
                MapCommand::SetStyle {
                    handle: MapHandle(1),
                    style_url: "maplibre://styles/satellite".to_owned(),
                },
                MapCommand::Destroy {
                    handle: MapHandle(1),
                },
            ]
        );
    }

    #[test]
    fn controller_lifecycle_requires_handle_for_handle_bound_commands() {
        let mut controller = MapController::new(FakeTransport::new());

        let error = controller.resize().unwrap_err();

        assert_eq!(
            error.to_string(),
            "missing MapLibre handle: resize requires an initialized map handle"
        );
        assert!(controller.transport().commands().is_empty());
    }

    #[test]
    fn controller_lifecycle_keeps_handle_when_destroy_transport_fails() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));
        controller.transport_mut().fail_next("bridge closed");

        let error = controller.destroy().unwrap_err();

        assert_eq!(
            error.to_string(),
            "MapLibre transport failed: bridge closed"
        );
        assert_eq!(controller.handle(), Some(MapHandle(1)));
    }

    #[test]
    fn lifecycle_cleanup_controller_destroy_if_initialized_is_idempotent() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        assert!(controller.destroy_if_initialized().unwrap());
        assert!(!controller.destroy_if_initialized().unwrap());

        assert_eq!(controller.handle(), None);
        assert_eq!(
            controller.transport().commands(),
            &[MapCommand::Destroy {
                handle: MapHandle(1),
            }]
        );
    }

    #[test]
    fn lifecycle_cleanup_controller_resize_if_initialized_skips_missing_handle() {
        let mut controller = MapController::new(FakeTransport::new());

        assert!(!controller.resize_if_initialized().unwrap());
        assert!(controller.transport().commands().is_empty());

        controller.set_handle(MapHandle(1));
        assert!(controller.resize_if_initialized().unwrap());
        assert_eq!(
            controller.transport().commands(),
            &[MapCommand::Resize {
                handle: MapHandle(1),
            }]
        );
    }

    #[test]
    fn controller_camera_sends_camera_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .fly_to(
                LngLat {
                    lng: -123.1,
                    lat: 49.2,
                },
                Some(11.0),
                Some(750),
            )
            .unwrap();
        controller
            .jump_to(
                LngLat {
                    lng: -123.2,
                    lat: 49.3,
                },
                Some(12.0),
                Some(15.0),
                Some(30.0),
            )
            .unwrap();
        controller
            .ease_to(
                LngLat {
                    lng: -123.3,
                    lat: 49.4,
                },
                Some(13.0),
                None,
                Some(25.0),
                Some(500),
            )
            .unwrap();
        controller
            .fit_bounds(
                Bounds {
                    west: -124.0,
                    south: 48.0,
                    east: -122.0,
                    north: 50.0,
                },
                Some(24.0),
                None,
                Some(14.0),
            )
            .unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::FlyTo {
                    handle: MapHandle(1),
                    lng: -123.1,
                    lat: 49.2,
                    zoom: Some(11.0),
                    duration_ms: Some(750),
                },
                MapCommand::JumpTo {
                    handle: MapHandle(1),
                    lng: -123.2,
                    lat: 49.3,
                    zoom: Some(12.0),
                    bearing: Some(15.0),
                    pitch: Some(30.0),
                },
                MapCommand::EaseTo {
                    handle: MapHandle(1),
                    lng: -123.3,
                    lat: 49.4,
                    zoom: Some(13.0),
                    bearing: None,
                    pitch: Some(25.0),
                    duration_ms: Some(500),
                },
                MapCommand::FitBounds {
                    handle: MapHandle(1),
                    west: -124.0,
                    south: 48.0,
                    east: -122.0,
                    north: 50.0,
                    padding: Some(24.0),
                    duration_ms: None,
                    max_zoom: Some(14.0),
                },
            ]
        );
    }

    #[test]
    fn controller_sources_sends_source_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .add_source(
                "tiles",
                json!({
                    "type": SourceType::Vector.as_str(),
                    "url": "mapbox://tiles"
                }),
            )
            .unwrap();
        controller
            .add_geojson_source(
                "places",
                json!({
                    "type": "FeatureCollection",
                    "features": []
                }),
                Some("id".to_owned()),
            )
            .unwrap();
        controller
            .update_geojson_source(
                "places",
                json!({
                    "type": "FeatureCollection",
                    "features": [{
                        "type": "Feature",
                        "id": "se-1"
                    }]
                }),
            )
            .unwrap();
        controller.remove_source("tiles").unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::AddSource {
                    handle: MapHandle(1),
                    source_id: "tiles".to_owned(),
                    source_spec: json!({
                        "type": "vector",
                        "url": "mapbox://tiles"
                    }),
                },
                MapCommand::AddGeoJsonSource {
                    handle: MapHandle(1),
                    source_id: "places".to_owned(),
                    geojson: json!({
                        "type": "FeatureCollection",
                        "features": []
                    }),
                    promote_id: Some("id".to_owned()),
                },
                MapCommand::UpdateGeoJsonSource {
                    handle: MapHandle(1),
                    source_id: "places".to_owned(),
                    geojson: json!({
                        "type": "FeatureCollection",
                        "features": [{
                            "type": "Feature",
                            "id": "se-1"
                        }]
                    }),
                },
                MapCommand::RemoveSource {
                    handle: MapHandle(1),
                    source_id: "tiles".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn controller_layers_sends_layer_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .add_layer(
                "places-circle",
                json!({
                    "type": LayerType::Circle.as_str(),
                    "source": "places"
                }),
                Some("labels".to_owned()),
            )
            .unwrap();
        controller
            .set_layout_property("places-circle", "visibility", json!("none"))
            .unwrap();
        controller
            .set_paint_property("places-circle", "circle-color", json!("#2b6cb0"))
            .unwrap();
        controller
            .set_filter(
                "places-circle",
                Some(json!(["==", ["get", "kind"], "harbor"])),
            )
            .unwrap();
        controller
            .set_layer_zoom_range("places-circle", Some(4.0), Some(12.0))
            .unwrap();
        controller.remove_layer("places-circle").unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::AddLayer {
                    handle: MapHandle(1),
                    layer_id: "places-circle".to_owned(),
                    layer_spec: json!({
                        "type": "circle",
                        "source": "places"
                    }),
                    before_id: Some("labels".to_owned()),
                },
                MapCommand::SetLayoutProperty {
                    handle: MapHandle(1),
                    layer_id: "places-circle".to_owned(),
                    property_name: "visibility".to_owned(),
                    value: json!("none"),
                },
                MapCommand::SetPaintProperty {
                    handle: MapHandle(1),
                    layer_id: "places-circle".to_owned(),
                    property_name: "circle-color".to_owned(),
                    value: json!("#2b6cb0"),
                },
                MapCommand::SetFilter {
                    handle: MapHandle(1),
                    layer_id: "places-circle".to_owned(),
                    filter: Some(json!(["==", ["get", "kind"], "harbor"])),
                },
                MapCommand::SetLayerZoomRange {
                    handle: MapHandle(1),
                    layer_id: "places-circle".to_owned(),
                    min_zoom: Some(4.0),
                    max_zoom: Some(12.0),
                },
                MapCommand::RemoveLayer {
                    handle: MapHandle(1),
                    layer_id: "places-circle".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn controller_feature_state_sends_feature_state_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .set_feature_state(
                "places",
                Some("settlements".to_owned()),
                json!("se-1"),
                json!({
                    "selected": true,
                    "hovered": false
                }),
            )
            .unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![MapCommand::SetFeatureState {
                handle: MapHandle(1),
                source_id: "places".to_owned(),
                source_layer: Some("settlements".to_owned()),
                feature_id: json!("se-1"),
                state: json!({
                    "selected": true,
                    "hovered": false
                }),
            }]
        );
    }

    #[test]
    fn controller_scene_options_sends_terrain_fog_and_light_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .set_terrain(Some(json!({
                "source": "terrain",
                "exaggeration": 1.2
            })))
            .unwrap();
        controller
            .set_fog(Some(json!({
                "range": [0.5, 10.0],
                "color": "#d7e7ff"
            })))
            .unwrap();
        controller
            .set_light(Some(json!({
                "anchor": "viewport",
                "intensity": 0.4
            })))
            .unwrap();
        controller.set_fog(None).unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::SetTerrain {
                    handle: MapHandle(1),
                    terrain: Some(json!({
                        "source": "terrain",
                        "exaggeration": 1.2
                    })),
                },
                MapCommand::SetFog {
                    handle: MapHandle(1),
                    fog: Some(json!({
                        "range": [0.5, 10.0],
                        "color": "#d7e7ff"
                    })),
                },
                MapCommand::SetLight {
                    handle: MapHandle(1),
                    light: Some(json!({
                        "anchor": "viewport",
                        "intensity": 0.4
                    })),
                },
                MapCommand::SetFog {
                    handle: MapHandle(1),
                    fog: None,
                },
            ]
        );
    }

    #[test]
    fn controller_controls_sends_control_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        let request_id = controller
            .add_native_control(
                NativeControlKind::Navigation,
                Some(MapControlAnchor::TopRight),
                Some(json!({"showCompass": true})),
            )
            .unwrap();
        controller.remove_native_control(ControlHandle(9)).unwrap();

        assert_eq!(request_id, 1);
        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::AddNativeControl {
                    request_id: 1,
                    handle: MapHandle(1),
                    kind: NativeControlKind::Navigation,
                    anchor: Some(MapControlAnchor::TopRight),
                    options: Some(json!({"showCompass": true})),
                },
                MapCommand::RemoveNativeControl {
                    control_handle: ControlHandle(9),
                },
            ]
        );
    }

    #[test]
    fn controller_markers_sends_marker_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        let request_id = controller
            .create_marker(MarkerOptions {
                draggable: true,
                ..marker_options(-123.1, 49.2)
            })
            .unwrap();
        controller
            .update_marker(
                MarkerHandle(10),
                MarkerOptions {
                    rotation: Some(45.0),
                    ..marker_options(-123.2, 49.3)
                },
            )
            .unwrap();
        controller.remove_marker(MarkerHandle(10)).unwrap();

        assert_eq!(request_id, 1);
        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::CreateMarker {
                    request_id: 1,
                    handle: MapHandle(1),
                    options: MarkerOptions {
                        draggable: true,
                        ..marker_options(-123.1, 49.2)
                    },
                },
                MapCommand::UpdateMarker {
                    marker_handle: MarkerHandle(10),
                    options: MarkerOptions {
                        rotation: Some(45.0),
                        ..marker_options(-123.2, 49.3)
                    },
                },
                MapCommand::RemoveMarker {
                    marker_handle: MarkerHandle(10),
                },
            ]
        );
    }

    #[test]
    fn controller_popups_sends_popup_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        let request_id = controller
            .create_popup(popup_options(-123.1, 49.2, "Harbor"))
            .unwrap();
        controller
            .update_popup(PopupHandle(11), popup_options(-123.2, 49.3, "Updated"))
            .unwrap();
        controller.remove_popup(PopupHandle(11)).unwrap();

        assert_eq!(request_id, 1);
        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::CreatePopup {
                    request_id: 1,
                    handle: MapHandle(1),
                    options: popup_options(-123.1, 49.2, "Harbor"),
                },
                MapCommand::UpdatePopup {
                    popup_handle: PopupHandle(11),
                    options: popup_options(-123.2, 49.3, "Updated"),
                },
                MapCommand::RemovePopup {
                    popup_handle: PopupHandle(11),
                },
            ]
        );
    }

    #[test]
    fn request_ids_increment_after_successful_request_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        assert_eq!(controller.next_request_id(), 1);
        assert_eq!(
            controller
                .create_marker(marker_options(-123.1, 49.2))
                .unwrap(),
            1
        );
        assert_eq!(controller.next_request_id(), 2);

        controller.transport_mut().fail_next("bridge closed");
        let error = controller
            .create_popup(popup_options(-123.2, 49.3, "Retry me"))
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "MapLibre transport failed: bridge closed"
        );
        assert_eq!(controller.next_request_id(), 2);

        assert_eq!(
            controller
                .create_popup(popup_options(-123.2, 49.3, "Retry me"))
                .unwrap(),
            2
        );
        assert_eq!(controller.next_request_id(), 3);
    }

    #[test]
    fn subscriptions_sends_subscription_commands() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .subscribe_map_events(MapEventSubscription::default().with_throttle_ms(250))
            .unwrap();
        controller.unsubscribe_map_events().unwrap();
        controller
            .subscribe_layer_events(LayerEventSubscription::clicks("places"))
            .unwrap();
        controller.unsubscribe_layer_events("places").unwrap();
        controller
            .subscribe_marker_drag_events(
                MarkerHandle(10),
                MarkerDragEventSubscription::default().with_throttle_ms(16),
            )
            .unwrap();
        controller
            .unsubscribe_marker_drag_events(MarkerHandle(10))
            .unwrap();
        controller
            .subscribe_popup_events(PopupHandle(11), PopupEventSubscription::default())
            .unwrap();
        controller
            .unsubscribe_popup_events(PopupHandle(11))
            .unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::SubscribeMapEvents {
                    handle: MapHandle(1),
                    subscription: MapEventSubscription::default().with_throttle_ms(250),
                },
                MapCommand::UnsubscribeMapEvents {
                    handle: MapHandle(1),
                },
                MapCommand::SubscribeLayerEvents {
                    handle: MapHandle(1),
                    subscription: LayerEventSubscription::clicks("places"),
                },
                MapCommand::UnsubscribeLayerEvents {
                    handle: MapHandle(1),
                    layer_id: "places".to_owned(),
                },
                MapCommand::SubscribeMarkerDragEvents {
                    marker_handle: MarkerHandle(10),
                    subscription: MarkerDragEventSubscription::default().with_throttle_ms(16),
                },
                MapCommand::UnsubscribeMarkerDragEvents {
                    marker_handle: MarkerHandle(10),
                },
                MapCommand::SubscribePopupEvents {
                    popup_handle: PopupHandle(11),
                    subscription: PopupEventSubscription::default(),
                },
                MapCommand::UnsubscribePopupEvents {
                    popup_handle: PopupHandle(11),
                },
            ]
        );
    }

    #[test]
    fn subscriptions_controller_accepts_generic_event_api() {
        let mut controller = MapController::with_handle(FakeTransport::new(), MapHandle(1));

        controller
            .subscribe_events(EventSubscription::map(MapEventSubscription::default()))
            .unwrap();
        controller
            .subscribe_events(EventSubscription::layer(LayerEventSubscription::clicks(
                "places",
            )))
            .unwrap();
        controller
            .subscribe_events(EventSubscription::marker_drag(
                MarkerHandle(10),
                MarkerDragEventSubscription::default(),
            ))
            .unwrap();
        controller
            .subscribe_events(EventSubscription::popup(
                PopupHandle(11),
                PopupEventSubscription::default(),
            ))
            .unwrap();
        controller
            .unsubscribe_events(EventSubscriptionTarget::layer("places"))
            .unwrap();
        controller
            .unsubscribe_events(EventSubscriptionTarget::marker_drag(MarkerHandle(10)))
            .unwrap();

        assert_eq!(
            controller.into_transport().into_commands(),
            vec![
                MapCommand::SubscribeMapEvents {
                    handle: MapHandle(1),
                    subscription: MapEventSubscription::default(),
                },
                MapCommand::SubscribeLayerEvents {
                    handle: MapHandle(1),
                    subscription: LayerEventSubscription::clicks("places"),
                },
                MapCommand::SubscribeMarkerDragEvents {
                    marker_handle: MarkerHandle(10),
                    subscription: MarkerDragEventSubscription::default(),
                },
                MapCommand::SubscribePopupEvents {
                    popup_handle: PopupHandle(11),
                    subscription: PopupEventSubscription::default(),
                },
                MapCommand::UnsubscribeLayerEvents {
                    handle: MapHandle(1),
                    layer_id: "places".to_owned(),
                },
                MapCommand::UnsubscribeMarkerDragEvents {
                    marker_handle: MarkerHandle(10),
                },
            ]
        );
    }
}
