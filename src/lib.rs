pub mod aggregate;
pub mod command;
pub mod command_bus;
pub mod command_handler;
pub mod event;
pub mod event_bus;
pub mod event_bus_rabbit;
pub mod event_handler;
pub mod event_metadata;
pub mod event_store;
pub mod event_store_postgres;
pub mod projections;
pub mod query;
pub mod query_bus;
pub mod query_handler;
pub mod snapshot;

pub use aggregate::Aggregate;
pub use command::Command;
pub use command_bus::{CommandBus, InMemoryCommandBus};
pub use command_handler::{CommandHandler, CommandHandlerError};
pub use event::Event;
pub use event_bus::{EventBus, InMemoryEventBus};
pub use event_bus_rabbit::RabbitEventBus;
pub use event_handler::{EventHandler, ProjectionEventHandler};
pub use event_metadata::{EventEnvelope, EventMetadata};
pub use event_store::EventStore;
pub use event_store_postgres::{Migrator, PostgresEventStore};
pub use projections::Projection;
pub use query::Query;
pub use query_bus::{InMemoryQueryBus, QueryBus};
pub use query_handler::QueryHandler;
pub use snapshot::{Snapshot, SnapshotStore};

#[derive(Debug)]
pub enum FrameworkError {
    EventStore(Box<dyn std::error::Error + Send + Sync>),
    Migration(Box<dyn std::error::Error + Send + Sync>),
}

pub struct Framework<E: Event> {
    pub event_store: Box<dyn EventStore<E, String, Error = FrameworkError> + Send + Sync>,
    migrator: Box<dyn Migrator + Send + Sync>,
}

impl<E: Event> Framework<E> {
    pub async fn setup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { self.migrator.migrate().await }
}
