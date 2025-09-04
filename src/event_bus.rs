use async_trait::async_trait;

use crate::{Event, EventEnvelope, EventHandler};

#[async_trait]
pub trait EventBus<E: Event> {
    type Error;

    async fn publish(&self, events: &[EventEnvelope<E>]) -> Result<(), Self::Error>;
    fn subscribe<H: EventHandler<E> + Send + Sync + 'static>(&mut self, handler: H);
}

#[async_trait]
trait ErasedEventHandler<E: Event>: Send + Sync {
    async fn handle(&self, event: &EventEnvelope<E>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<E: Event, H: EventHandler<E> + Send + Sync> ErasedEventHandler<E> for H {
    async fn handle(&self, event: &EventEnvelope<E>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        EventHandler::handle(self, event)
            .await
            .map_err(|_| "Event handler failed".into())
    }
}

pub struct InMemoryEventBus<E: Event> {
    handlers: Vec<Box<dyn ErasedEventHandler<E>>>,
}

impl<E: Event> Default for InMemoryEventBus<E> {
    fn default() -> Self { Self::new() }
}

impl<E: Event> InMemoryEventBus<E> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

#[async_trait]
impl<E: Event> EventBus<E> for InMemoryEventBus<E> {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn publish(&self, events: &[EventEnvelope<E>]) -> Result<(), Self::Error> {
        log::info!("Publishing {} events to {} handlers", events.len(), self.handlers.len());

        for event in events {
            log::debug!("Processing event: {}", event.event.event_type());

            for (i, handler) in self.handlers.iter().enumerate() {
                log::debug!("Calling handler {}", i);

                handler.handle(event).await?;
            }
        }

        log::debug!("All events published successfully");

        Ok(())
    }

    fn subscribe<H: EventHandler<E> + Send + Sync + 'static>(&mut self, handler: H) {
        log::info!("Registering new event handler (total: {})", self.handlers.len() + 1);

        self.handlers.push(Box::new(handler));
    }
}
