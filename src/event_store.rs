use crate::Event;

#[async_trait::async_trait]
pub trait EventStore<E: Event> {
    type Error;
    async fn save_event(
        &self, aggregate_id: &E::AggregateId, events: Vec<E>, expected_version: u64,
    ) -> Result<(), Self::Error>;
    async fn get_events(&self, aggregate_id: &E::AggregateId) -> Result<Vec<E>, Self::Error>;
}
