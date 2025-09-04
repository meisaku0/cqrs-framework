use async_trait::async_trait;

use crate::Query;

#[async_trait]
pub trait QueryHandler<Q: Query> {
    type Error;
    async fn handle(&self, query: Q) -> Result<Q::Result, Self::Error>;
}
