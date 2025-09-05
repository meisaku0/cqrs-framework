use async_trait::async_trait;
use lapin::options::*;
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use serde::Serialize;

use crate::{Event, EventBus, EventEnvelope};

#[derive(Clone)]
pub struct RabbitEventBus {
    channel: Channel,
    exchange: String,
}

impl RabbitEventBus {
    pub async fn new(amqp_url: &str, exchange: String) -> Result<Self, lapin::Error> {
        let conn = Connection::connect(amqp_url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

        channel
            .exchange_declare(
                &exchange,
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            channel,
            exchange,
        })
    }
}

#[async_trait]
impl<E: Event + Serialize + Send + Sync> EventBus<E> for RabbitEventBus {
    type Error = lapin::Error;

    async fn publish(&self, events: &[EventEnvelope<E>]) -> Result<(), Self::Error> {
        log::info!(
            "Publishing {} events to RabbitMQ exchange: {}",
            events.len(),
            self.exchange
        );

        for event in events {
            let payload = serde_json::to_string(event).unwrap().into_bytes();
            let routing_key = format!("events.{}", event.event.event_type());

            self.channel
                .basic_publish(
                    &self.exchange,
                    &routing_key,
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default(),
                )
                .await?;

            log::debug!("Published event: {}", event.event.event_type());
        }

        Ok(())
    }

    fn subscribe<H: crate::EventHandler<E> + Send + Sync + 'static>(&mut self, _handler: H) {
        log::warn!("RabbitMQ subscribe not implemented - use separate consumer");
    }
}
