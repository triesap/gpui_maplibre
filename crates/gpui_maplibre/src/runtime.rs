use crate::event::MapLibreEvent;
use crate::ids::{ControlHandle, MapHandle, MarkerHandle, PopupHandle};
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
                    .insert(request_id, control_handle);
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
                    .insert(request_id, marker_handle);
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
                    .insert(request_id, popup_handle);
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
}
