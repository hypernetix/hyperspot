use modkit::SseBroadcaster;

use crate::domain::{events::UserDomainEvent, ports::EventPublisher};

use super::dto::UserEvent;

/// Adapter: implements domain port and forwards events into SSE broadcaster.
pub struct SseUserEventPublisher {
    out: SseBroadcaster<UserEvent>,
}

impl SseUserEventPublisher {
    pub fn new(out: SseBroadcaster<UserEvent>) -> Self {
        Self { out }
    }
}

impl EventPublisher<UserDomainEvent> for SseUserEventPublisher {
    fn publish(&self, event: &UserDomainEvent) {
        self.out.send(UserEvent::from(event));
    }
}
