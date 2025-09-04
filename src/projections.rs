use async_trait::async_trait;

#[async_trait]
pub trait Projection: Send + Sync {
    type Event;
    type Error;

    async fn apply(&mut self, event: &Self::Event) -> Result<(), Self::Error>;
}
