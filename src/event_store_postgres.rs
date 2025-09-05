use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::{Event, EventEnvelope, EventMetadata, EventStore};

#[derive(Clone)]
pub struct PostgresEventStore {
    pub(crate) pool: PgPool,
}

#[derive(Debug)]
pub enum PostgresError {
    Sqlx(sqlx::Error),
    Serialization(serde_json::Error),
    ConcurrencyConflict,
}

impl From<sqlx::Error> for PostgresError {
    fn from(err: sqlx::Error) -> Self { PostgresError::Sqlx(err) }
}

impl From<serde_json::Error> for PostgresError {
    fn from(err: serde_json::Error) -> Self { PostgresError::Serialization(err) }
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
        }
    }
}

#[async_trait]
impl<E: Event + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static> EventStore<E, String>
    for PostgresEventStore
{
    type Error = PostgresError;

    async fn save_events(
        &self, aggregate_id: &String, events: Vec<EventEnvelope<E>>, expected_version: u64,
    ) -> Result<(), Self::Error> {
        let mut tx = self.pool.begin().await?;

        let current_version: Option<i64> =
            sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM events WHERE aggregate_id = $1")
                .bind(aggregate_id)
                .fetch_one(&mut *tx)
                .await?;

        if current_version.unwrap_or(0) != expected_version as i64 {
            return Err(PostgresError::ConcurrencyConflict);
        }

        for (i, envelope) in events.iter().enumerate() {
            let version = expected_version + i as u64 + 1;
            let event_data = serde_json::to_value(&envelope.event)?;
            let metadata = serde_json::to_value(&envelope.metadata)?;

            sqlx::query(
                "INSERT INTO events (aggregate_id, event_type, event_data, metadata, version) VALUES ($1, $2, $3, $4, \
                 $5)",
            )
            .bind(aggregate_id)
            .bind(envelope.event.event_type())
            .bind(event_data)
            .bind(metadata)
            .bind(version as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_events(&self, aggregate_id: &String) -> Result<Vec<EventEnvelope<E>>, Self::Error> {
        let rows = sqlx::query("SELECT event_data, metadata FROM events WHERE aggregate_id = $1 ORDER BY version")
            .bind(aggregate_id)
            .fetch_all(&self.pool)
            .await?;

        let mut events = Vec::new();
        for row in rows {
            let event: E = serde_json::from_value(row.get("event_data"))?;
            let metadata: EventMetadata = serde_json::from_value(row.get("metadata"))?;
            events.push(EventEnvelope {
                event,
                metadata,
            });
        }

        Ok(events)
    }

    async fn get_events_from_version(
        &self, aggregate_id: &String, from_version: u64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error> {
        let rows = sqlx::query(
            "SELECT event_data, metadata FROM events WHERE aggregate_id = $1 AND version > $2 ORDER BY version",
        )
        .bind(aggregate_id)
        .bind(from_version as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let event: E = serde_json::from_value(row.get("event_data"))?;
            let metadata: EventMetadata = serde_json::from_value(row.get("metadata"))?;
            events.push(EventEnvelope {
                event,
                metadata,
            });
        }

        Ok(events)
    }
}

#[async_trait]
pub trait Migrator {
    async fn migrate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl Migrator for PostgresEventStore {
    async fn migrate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            aggregate_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            event_data JSONB NOT NULL,
            metadata JSONB NOT NULL,
            version BIGINT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_aggregate_id ON events(aggregate_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_version ON events(aggregate_id, version)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
