use std::any::TypeId;
use std::collections::HashMap;

use async_trait::async_trait;

use crate::{Query, QueryHandler};

#[async_trait]
pub trait QueryBus {
    type Error;

    async fn send<Q: Query + 'static>(&self, query: Q) -> Result<Q::Result, Self::Error>;
}

#[async_trait]
pub trait ErasedQueryHandler: Send + Sync {
    async fn handle(
        &self, query: Box<dyn std::any::Any + Send>,
    ) -> Result<Box<dyn std::any::Any + Send>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct TypedQueryHandler<Q: Query, H: QueryHandler<Q>> {
    handler: H,
    _phantom: std::marker::PhantomData<Q>,
}

impl<Q: Query, H: QueryHandler<Q>> TypedQueryHandler<Q, H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<Q: Query + 'static, H: QueryHandler<Q> + Send + Sync> ErasedQueryHandler for TypedQueryHandler<Q, H> {
    async fn handle(
        &self, query: Box<dyn std::any::Any + Send>,
    ) -> Result<Box<dyn std::any::Any + Send>, Box<dyn std::error::Error + Send + Sync>> {
        let typed_query = *query.downcast::<Q>().map_err(|_| "Type mismatch")?;
        let result = self
            .handler
            .handle(typed_query)
            .await
            .map_err(|_| "Query execution failed")?;
        Ok(Box::new(result))
    }
}

pub struct InMemoryQueryBus {
    handlers: HashMap<TypeId, Box<dyn ErasedQueryHandler>>,
}

impl Default for InMemoryQueryBus {
    fn default() -> Self { Self::new() }
}

impl InMemoryQueryBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<Q, H>(&mut self, handler: H)
    where
        Q: Query + 'static,
        H: QueryHandler<Q> + Send + Sync + 'static,
    {
        let wrapped = TypedQueryHandler::new(handler);
        self.handlers.insert(TypeId::of::<Q>(), Box::new(wrapped));
    }
}

#[async_trait]
impl QueryBus for InMemoryQueryBus {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn send<Q: Query + 'static>(&self, query: Q) -> Result<Q::Result, Self::Error> {
        let type_id = TypeId::of::<Q>();
        let handler = self.handlers.get(&type_id).ok_or("No handler registered for query")?;

        let result = handler.handle(Box::new(query)).await?;
        let typed_result = *result.downcast::<Q::Result>().map_err(|_| "Result type mismatch")?;
        Ok(typed_result)
    }
}
