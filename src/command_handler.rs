use async_trait::async_trait;

use crate::{Aggregate, Command, EventEnvelope, EventMetadata, EventStore, SnapshotStore};

pub trait CommandHandlerError {
    fn from_event_store_error<E>(err: E) -> Self;
    fn from_command_error<E>(err: E) -> Self;
}

#[async_trait]
pub trait CommandHandler<C: Command> {
    type EventStore: EventStore<<C::Aggregate as Aggregate>::Event, C::AggregateId>;
    type SnapshotStore: SnapshotStore<C::Aggregate, C::AggregateId>;
    type Error: CommandHandlerError;

    fn event_store(&self) -> &Self::EventStore;
    fn snapshot_store(&self) -> &Self::SnapshotStore;

    async fn handle(&self, command: C) -> Result<(), Self::Error>
    where
        C: 'static,
    {
        let snapshot = self
            .snapshot_store()
            .get_snapshot(command.aggregate_id())
            .await
            .ok()
            .flatten();

        let mut aggregate = snapshot.unwrap_or_default();
        let from_version = aggregate.version();

        let events = self
            .event_store()
            .get_events_from_version(command.aggregate_id(), from_version)
            .await
            .map_err(Self::Error::from_event_store_error)?;

        for envelope in events {
            aggregate.apply(envelope.event);
        }

        let new_events = command.execute(&aggregate).map_err(Self::Error::from_command_error)?;

        if !new_events.is_empty() {
            let correlation_id = uuid::Uuid::new_v4();
            let envelopes: Vec<_> = new_events
                .into_iter()
                .map(|event| {
                    EventEnvelope {
                        event,
                        metadata: EventMetadata::new(correlation_id, None),
                    }
                })
                .collect();

            let envelope_count = envelopes.len() as u64;

            self.event_store()
                .save_events(command.aggregate_id(), envelopes, aggregate.version())
                .await
                .map_err(Self::Error::from_event_store_error)?;

            if (aggregate.version() + envelope_count) % 10 == 0 {
                self.snapshot_store()
                    .save_snapshot(command.aggregate_id(), aggregate)
                    .await
                    .ok();
            }
        }

        Ok(())
    }
}
