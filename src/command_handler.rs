use async_trait::async_trait;
use uuid::Uuid;

use crate::{Aggregate, Command, EventEnvelope, EventMetadata, EventStore};

pub trait CommandHandlerError {
    fn from_event_store_error<E>(err: E) -> Self;
    fn from_command_error<E>(err: E) -> Self;
}

#[async_trait]
pub trait CommandHandler<C: Command> {
    type EventStore: EventStore<<C::Aggregate as Aggregate>::Event, C::AggregateId>;

    type Error: CommandHandlerError;

    fn event_store(&self) -> &Self::EventStore;

    async fn handle(&self, command: C) -> Result<(), Self::Error>
    where
        C: 'static,
    {
        let event_envelopes = self
            .event_store()
            .get_events(command.aggregate_id())
            .await
            .map_err(Self::Error::from_event_store_error)?;

        let mut aggregate = C::Aggregate::default();
        for envelope in event_envelopes {
            aggregate.apply(envelope.event);
        }

        let new_events = command.execute(&aggregate).map_err(Self::Error::from_command_error)?;

        if !new_events.is_empty() {
            let correlation_id = Uuid::new_v4();
            let envelopes: Vec<_> = new_events
                .into_iter()
                .map(|event| {
                    EventEnvelope {
                        event,
                        metadata: EventMetadata::new(correlation_id, None),
                    }
                })
                .collect();

            self.event_store()
                .save_events(command.aggregate_id(), envelopes, aggregate.version())
                .await
                .map_err(Self::Error::from_event_store_error)?;
        }

        Ok(())
    }
}
