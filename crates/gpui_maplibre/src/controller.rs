use crate::ids::MapHandle;
use crate::options::MapInitOptions;
use crate::transport::CommandTransport;
use crate::types::{Bounds, LngLat};
use crate::{MapCommand, MapLibreError, Result};

#[derive(Clone, Debug)]
pub struct MapController<T> {
    transport: T,
    handle: Option<MapHandle>,
}

impl<T> MapController<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            handle: None,
        }
    }

    pub fn with_handle(transport: T, handle: MapHandle) -> Self {
        Self {
            transport,
            handle: Some(handle),
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

    pub fn resize(&mut self) -> Result<()> {
        let handle = self.require_handle("resize requires an initialized map handle")?;

        self.send(MapCommand::Resize { handle })
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

    fn send(&mut self, command: MapCommand) -> Result<()> {
        self.transport.send_command(command)
    }

    fn require_handle(&self, context: &'static str) -> Result<MapHandle> {
        self.handle
            .ok_or_else(|| MapLibreError::missing_handle(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FakeTransport;

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
}
