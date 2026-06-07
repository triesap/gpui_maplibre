use crate::event::{MapLibreEvent, parse_ipc_event};
use crate::ids::{ControlHandle, MapHandle, MarkerHandle, PopupHandle};
use crate::{MapCommand, Result};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutedError {
    pub context: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventRouterAction {
    DomReady,
    Initialized {
        handle: MapHandle,
    },
    Ready {
        handle: MapHandle,
    },
    NativeControlCreated {
        request_id: u64,
        control_handle: ControlHandle,
        was_pending: bool,
    },
    MarkerCreated {
        request_id: u64,
        marker_handle: MarkerHandle,
        was_pending: bool,
    },
    PopupCreated {
        request_id: u64,
        popup_handle: PopupHandle,
        was_pending: bool,
    },
    Emit(MapLibreEvent),
    Error(RoutedError),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeCommandAction {
    Dispatch(MapCommand),
    Queued,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeCommandQueue {
    dom_ready: bool,
    initialized_handle: Option<MapHandle>,
    ready_handle: Option<MapHandle>,
    pending_commands: VecDeque<MapCommand>,
}

impl RuntimeCommandQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_dom_ready(&self) -> bool {
        self.dom_ready
    }

    pub fn initialized_handle(&self) -> Option<MapHandle> {
        self.initialized_handle
    }

    pub fn ready_handle(&self) -> Option<MapHandle> {
        self.ready_handle
    }

    pub fn map_handle(&self) -> Option<MapHandle> {
        self.ready_handle.or(self.initialized_handle)
    }

    pub fn is_map_ready(&self) -> bool {
        self.ready_handle.is_some()
    }

    pub fn pending_len(&self) -> usize {
        self.pending_commands.len()
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &MapCommand> {
        self.pending_commands.iter()
    }

    pub fn submit_command(&mut self, command: MapCommand) -> RuntimeCommandAction {
        if self.can_dispatch(&command) {
            RuntimeCommandAction::Dispatch(command)
        } else {
            self.pending_commands.push_back(command);
            RuntimeCommandAction::Queued
        }
    }

    pub fn reduce_event(&mut self, event: &MapLibreEvent) -> Vec<MapCommand> {
        match event {
            MapLibreEvent::DomReady => {
                self.dom_ready = true;
                self.drain_dispatchable_commands()
            }
            MapLibreEvent::Initialized { handle } => {
                self.initialized_handle = Some(*handle);
                Vec::new()
            }
            MapLibreEvent::Ready { handle } => {
                self.initialized_handle.get_or_insert(*handle);
                self.ready_handle = Some(*handle);
                self.drain_dispatchable_commands()
            }
            MapLibreEvent::Error { .. }
            | MapLibreEvent::Click { .. }
            | MapLibreEvent::Map { .. }
            | MapLibreEvent::Layer { .. }
            | MapLibreEvent::NativeControlCreated { .. }
            | MapLibreEvent::MarkerCreated { .. }
            | MapLibreEvent::MarkerDrag { .. }
            | MapLibreEvent::PopupCreated { .. }
            | MapLibreEvent::Popup { .. } => Vec::new(),
        }
    }

    fn can_dispatch(&self, command: &MapCommand) -> bool {
        match command {
            MapCommand::Init { .. } => self.dom_ready,
            MapCommand::Destroy { .. } => self.map_handle().is_some(),
            MapCommand::Resize { .. }
            | MapCommand::SetStyle { .. }
            | MapCommand::FlyTo { .. }
            | MapCommand::JumpTo { .. }
            | MapCommand::EaseTo { .. }
            | MapCommand::FitBounds { .. }
            | MapCommand::AddSource { .. }
            | MapCommand::AddGeoJsonSource { .. }
            | MapCommand::UpdateGeoJsonSource { .. }
            | MapCommand::RemoveSource { .. }
            | MapCommand::AddLayer { .. }
            | MapCommand::RemoveLayer { .. }
            | MapCommand::SetLayoutProperty { .. }
            | MapCommand::SetPaintProperty { .. }
            | MapCommand::SetFilter { .. }
            | MapCommand::SetLayerZoomRange { .. }
            | MapCommand::SetFeatureState { .. }
            | MapCommand::SetTerrain { .. }
            | MapCommand::SetFog { .. }
            | MapCommand::SetLight { .. }
            | MapCommand::AddNativeControl { .. }
            | MapCommand::RemoveNativeControl { .. }
            | MapCommand::CreateMarker { .. }
            | MapCommand::UpdateMarker { .. }
            | MapCommand::RemoveMarker { .. }
            | MapCommand::CreatePopup { .. }
            | MapCommand::UpdatePopup { .. }
            | MapCommand::RemovePopup { .. }
            | MapCommand::SubscribeMapEvents { .. }
            | MapCommand::UnsubscribeMapEvents { .. }
            | MapCommand::SubscribeLayerEvents { .. }
            | MapCommand::UnsubscribeLayerEvents { .. }
            | MapCommand::SubscribeMarkerDragEvents { .. }
            | MapCommand::UnsubscribeMarkerDragEvents { .. }
            | MapCommand::SubscribePopupEvents { .. }
            | MapCommand::UnsubscribePopupEvents { .. } => self.is_map_ready(),
        }
    }

    fn drain_dispatchable_commands(&mut self) -> Vec<MapCommand> {
        let mut remaining = VecDeque::new();
        let mut dispatchable = Vec::new();

        while let Some(command) = self.pending_commands.pop_front() {
            if self.can_dispatch(&command) {
                dispatchable.push(command);
            } else {
                remaining.push_back(command);
            }
        }

        self.pending_commands = remaining;
        dispatchable
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventRouter {
    dom_ready: bool,
    initialized_handle: Option<MapHandle>,
    ready_handle: Option<MapHandle>,
    pending_control_requests: HashSet<u64>,
    pending_marker_requests: HashSet<u64>,
    pending_popup_requests: HashSet<u64>,
    control_handles_by_request: HashMap<u64, ControlHandle>,
    marker_handles_by_request: HashMap<u64, MarkerHandle>,
    popup_handles_by_request: HashMap<u64, PopupHandle>,
    errors: Vec<RoutedError>,
}

pub fn route_ipc_message(router: &mut EventRouter, message: &str) -> Result<EventRouterAction> {
    let event = parse_ipc_event(message)?;
    Ok(router.route_event(event))
}

impl EventRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_dom_ready(&self) -> bool {
        self.dom_ready
    }

    pub fn initialized_handle(&self) -> Option<MapHandle> {
        self.initialized_handle
    }

    pub fn ready_handle(&self) -> Option<MapHandle> {
        self.ready_handle
    }

    pub fn map_handle(&self) -> Option<MapHandle> {
        self.ready_handle.or(self.initialized_handle)
    }

    pub fn is_map_ready(&self) -> bool {
        self.ready_handle.is_some()
    }

    pub fn errors(&self) -> &[RoutedError] {
        &self.errors
    }

    pub fn track_control_request(&mut self, request_id: u64) {
        self.pending_control_requests.insert(request_id);
    }

    pub fn track_marker_request(&mut self, request_id: u64) {
        self.pending_marker_requests.insert(request_id);
    }

    pub fn track_popup_request(&mut self, request_id: u64) {
        self.pending_popup_requests.insert(request_id);
    }

    pub fn is_control_request_pending(&self, request_id: u64) -> bool {
        self.pending_control_requests.contains(&request_id)
    }

    pub fn is_marker_request_pending(&self, request_id: u64) -> bool {
        self.pending_marker_requests.contains(&request_id)
    }

    pub fn is_popup_request_pending(&self, request_id: u64) -> bool {
        self.pending_popup_requests.contains(&request_id)
    }

    pub fn control_handle_for_request(&self, request_id: u64) -> Option<ControlHandle> {
        self.control_handles_by_request.get(&request_id).copied()
    }

    pub fn marker_handle_for_request(&self, request_id: u64) -> Option<MarkerHandle> {
        self.marker_handles_by_request.get(&request_id).copied()
    }

    pub fn popup_handle_for_request(&self, request_id: u64) -> Option<PopupHandle> {
        self.popup_handles_by_request.get(&request_id).copied()
    }

    pub fn route_event(&mut self, event: MapLibreEvent) -> EventRouterAction {
        match event {
            MapLibreEvent::DomReady => {
                self.dom_ready = true;
                EventRouterAction::DomReady
            }
            MapLibreEvent::Initialized { handle } => {
                self.initialized_handle = Some(handle);
                EventRouterAction::Initialized { handle }
            }
            MapLibreEvent::Ready { handle } => {
                self.initialized_handle.get_or_insert(handle);
                self.ready_handle = Some(handle);
                EventRouterAction::Ready { handle }
            }
            MapLibreEvent::NativeControlCreated {
                request_id,
                control_handle,
            } => {
                let was_pending = self.pending_control_requests.remove(&request_id);
                self.control_handles_by_request
                    .entry(request_id)
                    .or_insert(control_handle);
                EventRouterAction::NativeControlCreated {
                    request_id,
                    control_handle,
                    was_pending,
                }
            }
            MapLibreEvent::MarkerCreated {
                request_id,
                marker_handle,
            } => {
                let was_pending = self.pending_marker_requests.remove(&request_id);
                self.marker_handles_by_request
                    .entry(request_id)
                    .or_insert(marker_handle);
                EventRouterAction::MarkerCreated {
                    request_id,
                    marker_handle,
                    was_pending,
                }
            }
            MapLibreEvent::PopupCreated {
                request_id,
                popup_handle,
            } => {
                let was_pending = self.pending_popup_requests.remove(&request_id);
                self.popup_handles_by_request
                    .entry(request_id)
                    .or_insert(popup_handle);
                EventRouterAction::PopupCreated {
                    request_id,
                    popup_handle,
                    was_pending,
                }
            }
            MapLibreEvent::Error { context, message } => {
                let error = RoutedError { context, message };
                self.errors.push(error.clone());
                EventRouterAction::Error(error)
            }
            event => EventRouterAction::Emit(event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        FeatureHit, MarkerDragEvent, MarkerDragEventKind, PopupLifecycleEvent,
        PopupLifecycleEventKind,
    };
    use serde_json::json;

    fn init_command() -> MapCommand {
        MapCommand::Init {
            options: crate::MapInitOptions::default(),
        }
    }

    fn resize_command() -> MapCommand {
        MapCommand::Resize {
            handle: MapHandle(1),
        }
    }

    #[test]
    fn event_router_updates_readiness_and_map_handle() {
        let mut router = EventRouter::new();

        assert!(!router.is_dom_ready());
        assert!(!router.is_map_ready());
        assert_eq!(router.map_handle(), None);

        assert_eq!(
            router.route_event(MapLibreEvent::DomReady),
            EventRouterAction::DomReady
        );
        assert!(router.is_dom_ready());

        assert_eq!(
            router.route_event(MapLibreEvent::Initialized {
                handle: MapHandle(1),
            }),
            EventRouterAction::Initialized {
                handle: MapHandle(1),
            }
        );
        assert_eq!(router.initialized_handle(), Some(MapHandle(1)));
        assert_eq!(router.map_handle(), Some(MapHandle(1)));
        assert!(!router.is_map_ready());

        assert_eq!(
            router.route_event(MapLibreEvent::Ready {
                handle: MapHandle(1),
            }),
            EventRouterAction::Ready {
                handle: MapHandle(1),
            }
        );
        assert_eq!(router.ready_handle(), Some(MapHandle(1)));
        assert!(router.is_map_ready());
    }

    #[test]
    fn event_router_resolves_pending_request_ids() {
        let mut router = EventRouter::new();
        router.track_control_request(10);
        router.track_marker_request(11);
        router.track_popup_request(12);

        assert_eq!(
            router.route_event(MapLibreEvent::NativeControlCreated {
                request_id: 10,
                control_handle: ControlHandle(2),
            }),
            EventRouterAction::NativeControlCreated {
                request_id: 10,
                control_handle: ControlHandle(2),
                was_pending: true,
            }
        );
        assert!(!router.is_control_request_pending(10));
        assert_eq!(
            router.control_handle_for_request(10),
            Some(ControlHandle(2))
        );

        assert_eq!(
            router.route_event(MapLibreEvent::MarkerCreated {
                request_id: 11,
                marker_handle: MarkerHandle(3),
            }),
            EventRouterAction::MarkerCreated {
                request_id: 11,
                marker_handle: MarkerHandle(3),
                was_pending: true,
            }
        );
        assert!(!router.is_marker_request_pending(11));
        assert_eq!(router.marker_handle_for_request(11), Some(MarkerHandle(3)));

        assert_eq!(
            router.route_event(MapLibreEvent::PopupCreated {
                request_id: 12,
                popup_handle: PopupHandle(4),
            }),
            EventRouterAction::PopupCreated {
                request_id: 12,
                popup_handle: PopupHandle(4),
                was_pending: true,
            }
        );
        assert!(!router.is_popup_request_pending(12));
        assert_eq!(router.popup_handle_for_request(12), Some(PopupHandle(4)));
    }

    #[test]
    fn event_router_marks_untracked_created_events_as_not_pending() {
        let mut router = EventRouter::new();

        assert_eq!(
            router.route_event(MapLibreEvent::MarkerCreated {
                request_id: 99,
                marker_handle: MarkerHandle(3),
            }),
            EventRouterAction::MarkerCreated {
                request_id: 99,
                marker_handle: MarkerHandle(3),
                was_pending: false,
            }
        );
        assert_eq!(router.marker_handle_for_request(99), Some(MarkerHandle(3)));
    }

    #[test]
    fn event_router_preserves_errors_for_inspection() {
        let mut router = EventRouter::new();

        let action = router.route_event(MapLibreEvent::Error {
            context: "add_layer".to_owned(),
            message: "duplicate id".to_owned(),
        });

        assert_eq!(
            action,
            EventRouterAction::Error(RoutedError {
                context: "add_layer".to_owned(),
                message: "duplicate id".to_owned(),
            })
        );
        assert_eq!(
            router.errors(),
            &[RoutedError {
                context: "add_layer".to_owned(),
                message: "duplicate id".to_owned(),
            }]
        );
    }

    #[test]
    fn event_router_emits_non_state_events_without_callbacks() {
        let mut router = EventRouter::new();

        let click = MapLibreEvent::Click {
            lng: -123.1,
            lat: 49.2,
            screen_x: 100.0,
            screen_y: 200.0,
            features: vec![FeatureHit {
                layer_id: "places".to_owned(),
                properties: json!({"name": "Harbor"}),
            }],
        };
        assert_eq!(
            router.route_event(click.clone()),
            EventRouterAction::Emit(click)
        );

        let marker_drag = MapLibreEvent::MarkerDrag {
            marker_handle: MarkerHandle(3),
            event: MarkerDragEvent {
                kind: MarkerDragEventKind::DragEnd,
                lng: -123.2,
                lat: 49.3,
            },
        };
        assert_eq!(
            router.route_event(marker_drag.clone()),
            EventRouterAction::Emit(marker_drag)
        );

        let popup = MapLibreEvent::Popup {
            popup_handle: PopupHandle(4),
            event: PopupLifecycleEvent {
                kind: PopupLifecycleEventKind::Open,
                lng: -123.3,
                lat: 49.4,
            },
        };
        assert_eq!(
            router.route_event(popup.clone()),
            EventRouterAction::Emit(popup)
        );
    }

    #[test]
    fn runtime_ready_queue_holds_init_until_dom_ready() {
        let mut queue = RuntimeCommandQueue::new();

        assert_eq!(
            queue.submit_command(init_command()),
            RuntimeCommandAction::Queued
        );
        assert_eq!(queue.pending_len(), 1);

        let dispatch = queue.reduce_event(&MapLibreEvent::DomReady);

        assert!(queue.is_dom_ready());
        assert_eq!(dispatch, vec![init_command()]);
        assert_eq!(queue.pending_len(), 0);
    }

    #[test]
    fn runtime_ready_queue_holds_map_commands_until_ready() {
        let mut queue = RuntimeCommandQueue::new();

        assert_eq!(
            queue.submit_command(resize_command()),
            RuntimeCommandAction::Queued
        );
        assert_eq!(
            queue.reduce_event(&MapLibreEvent::Initialized {
                handle: MapHandle(1),
            }),
            Vec::<MapCommand>::new()
        );
        assert_eq!(queue.initialized_handle(), Some(MapHandle(1)));
        assert_eq!(queue.pending_len(), 1);

        let dispatch = queue.reduce_event(&MapLibreEvent::Ready {
            handle: MapHandle(1),
        });

        assert!(queue.is_map_ready());
        assert_eq!(queue.ready_handle(), Some(MapHandle(1)));
        assert_eq!(dispatch, vec![resize_command()]);
        assert_eq!(queue.pending_len(), 0);
    }

    #[test]
    fn runtime_ready_queue_dispatches_immediately_after_ready() {
        let mut queue = RuntimeCommandQueue::new();
        queue.reduce_event(&MapLibreEvent::DomReady);
        queue.reduce_event(&MapLibreEvent::Ready {
            handle: MapHandle(1),
        });

        assert_eq!(
            queue.submit_command(init_command()),
            RuntimeCommandAction::Dispatch(init_command())
        );
        assert_eq!(
            queue.submit_command(resize_command()),
            RuntimeCommandAction::Dispatch(resize_command())
        );
        assert_eq!(queue.pending_len(), 0);
    }

    #[test]
    fn pending_handles_resolve_successful_request_ids() {
        let mut router = EventRouter::new();
        router.track_control_request(10);
        router.track_marker_request(11);
        router.track_popup_request(12);

        assert!(matches!(
            router.route_event(MapLibreEvent::NativeControlCreated {
                request_id: 10,
                control_handle: ControlHandle(7),
            }),
            EventRouterAction::NativeControlCreated {
                was_pending: true,
                ..
            }
        ));
        assert!(matches!(
            router.route_event(MapLibreEvent::MarkerCreated {
                request_id: 11,
                marker_handle: MarkerHandle(2),
            }),
            EventRouterAction::MarkerCreated {
                was_pending: true,
                ..
            }
        ));
        assert!(matches!(
            router.route_event(MapLibreEvent::PopupCreated {
                request_id: 12,
                popup_handle: PopupHandle(4),
            }),
            EventRouterAction::PopupCreated {
                was_pending: true,
                ..
            }
        ));

        assert_eq!(
            router.control_handle_for_request(10),
            Some(ControlHandle(7))
        );
        assert_eq!(router.marker_handle_for_request(11), Some(MarkerHandle(2)));
        assert_eq!(router.popup_handle_for_request(12), Some(PopupHandle(4)));
    }

    #[test]
    fn pending_handles_report_unknown_request_ids_without_pending_match() {
        let mut router = EventRouter::new();

        let action = router.route_event(MapLibreEvent::PopupCreated {
            request_id: 99,
            popup_handle: PopupHandle(4),
        });

        assert_eq!(
            action,
            EventRouterAction::PopupCreated {
                request_id: 99,
                popup_handle: PopupHandle(4),
                was_pending: false,
            }
        );
        assert_eq!(router.popup_handle_for_request(99), Some(PopupHandle(4)));
    }

    #[test]
    fn pending_handles_do_not_overwrite_duplicate_resolutions() {
        let mut router = EventRouter::new();
        router.track_marker_request(11);

        router.route_event(MapLibreEvent::MarkerCreated {
            request_id: 11,
            marker_handle: MarkerHandle(2),
        });
        let action = router.route_event(MapLibreEvent::MarkerCreated {
            request_id: 11,
            marker_handle: MarkerHandle(3),
        });

        assert_eq!(
            action,
            EventRouterAction::MarkerCreated {
                request_id: 11,
                marker_handle: MarkerHandle(3),
                was_pending: false,
            }
        );
        assert_eq!(router.marker_handle_for_request(11), Some(MarkerHandle(2)));
    }

    #[test]
    fn pending_handles_keep_control_marker_and_popup_maps_separate() {
        let mut router = EventRouter::new();

        router.route_event(MapLibreEvent::NativeControlCreated {
            request_id: 1,
            control_handle: ControlHandle(7),
        });
        router.route_event(MapLibreEvent::MarkerCreated {
            request_id: 1,
            marker_handle: MarkerHandle(2),
        });
        router.route_event(MapLibreEvent::PopupCreated {
            request_id: 1,
            popup_handle: PopupHandle(4),
        });

        assert_eq!(router.control_handle_for_request(1), Some(ControlHandle(7)));
        assert_eq!(router.marker_handle_for_request(1), Some(MarkerHandle(2)));
        assert_eq!(router.popup_handle_for_request(1), Some(PopupHandle(4)));
    }

    #[test]
    fn runtime_routes_ipc_messages_through_event_router() {
        let mut router = EventRouter::new();

        let action = route_ipc_message(&mut router, r#"{"type":"dom_ready"}"#).unwrap();
        assert_eq!(action, EventRouterAction::DomReady);
        assert!(router.is_dom_ready());

        let action =
            route_ipc_message(&mut router, r#"{"type":"initialized","handle":1}"#).unwrap();
        assert_eq!(
            action,
            EventRouterAction::Initialized {
                handle: MapHandle(1),
            }
        );
        assert_eq!(router.map_handle(), Some(MapHandle(1)));

        let error = route_ipc_message(&mut router, "{broken").unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("failed to parse MapLibre event:")
        );
    }
}
