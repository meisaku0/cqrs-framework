use async_trait::async_trait;

use crate::{Aggregate, Command, EventStore};

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
        let events = self
            .event_store()
            .get_events(command.aggregate_id())
            .await
            .map_err(Self::Error::from_event_store_error)?;

        let mut aggregate = C::Aggregate::default();
        for event in events {
            aggregate.apply(event);
        }

        let new_events = command.execute(&aggregate).map_err(Self::Error::from_command_error)?;

        if !new_events.is_empty() {
            self.event_store()
                .save_events(command.aggregate_id(), new_events, aggregate.version())
                .await
                .map_err(Self::Error::from_event_store_error)?;
        }

        Ok(())
    }
}
