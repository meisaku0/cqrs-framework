use async_trait::async_trait;

use crate::{Event, EventEnvelope};

#[async_trait]
pub trait EventStore<E: Event, Id> {
    type Error;

    async fn save_events(
        &self, aggregate_id: &Id, events: Vec<EventEnvelope<E>>, expected_version: u64,
    ) -> Result<(), Self::Error>;

    async fn get_events(&self, aggregate_id: &Id) -> Result<Vec<EventEnvelope<E>>, Self::Error>;

    async fn get_events_from_version(
        &self, aggregate_id: &Id, from_version: u64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error>;
}
