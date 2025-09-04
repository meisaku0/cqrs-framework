use async_trait::async_trait;

use crate::Event;

#[async_trait]
pub trait EventStore<E: Event, Id> {
    type Error;

    async fn save_events(&self, aggregate_id: &Id, events: Vec<E>, expected_version: u64) -> Result<(), Self::Error>;

    async fn get_events(&self, aggregate_id: &Id) -> Result<Vec<E>, Self::Error>;
}
