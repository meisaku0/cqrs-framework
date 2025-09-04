use async_trait::async_trait;

use crate::{Event, EventEnvelope, Projection};

#[async_trait]
pub trait EventHandler<E: Event> {
    type Error;

    async fn handle(&self, envelope: &EventEnvelope<E>) -> Result<(), Self::Error>;
}

pub struct ProjectionEventHandler<P> {
    projection: P,
}

impl<P> ProjectionEventHandler<P> {
    pub fn new(projection: P) -> Self {
        Self {
            projection,
        }
    }
}

#[async_trait]
impl<E: Event, P: Projection<Event = E> + Send + Sync> EventHandler<E> for ProjectionEventHandler<P> {
    type Error = P::Error;

    async fn handle(&self, envelope: &EventEnvelope<E>) -> Result<(), Self::Error> {
        log::debug!("Applying event to projection: {}", envelope.event.event_type());

        let result = self.projection.apply(&envelope.event).await;

        if result.is_ok() {
            log::debug!("Event applied successfully to projection");
        } else {
            log::warn!("Failed to apply event to projection");
        }
        result
    }
}
