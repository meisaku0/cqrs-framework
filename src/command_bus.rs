use std::any::TypeId;
use std::collections::HashMap;

use async_trait::async_trait;

use crate::{Command, CommandHandler};

#[async_trait]
pub trait CommandBus {
    type Error;

    async fn send<C: Command + 'static>(&self, command: C) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait ErasedCommandHandler: Send + Sync {
    async fn handle(
        &self, command: Box<dyn std::any::Any + Send>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct TypedCommandHandler<C: Command, H: CommandHandler<C>> {
    handler: H,
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Command, H: CommandHandler<C>> TypedCommandHandler<C, H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<C: Command + 'static, H: CommandHandler<C> + Send + Sync> ErasedCommandHandler for TypedCommandHandler<C, H> {
    async fn handle(
        &self, command: Box<dyn std::any::Any + Send>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let typed_command = *command.downcast::<C>().map_err(|_| "Type mismatch")?;
        self.handler
            .handle(typed_command)
            .await
            .map_err(|_| "Command execution failed".into())
    }
}

pub struct InMemoryCommandBus {
    handlers: HashMap<TypeId, Box<dyn ErasedCommandHandler>>,
}

impl Default for InMemoryCommandBus {
    fn default() -> Self { Self::new() }
}

impl InMemoryCommandBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<C, H>(&mut self, handler: H)
    where
        C: Command + 'static,
        H: CommandHandler<C> + Send + Sync + 'static,
    {
        let wrapped = TypedCommandHandler::new(handler);
        self.handlers.insert(TypeId::of::<C>(), Box::new(wrapped));
    }
}

#[async_trait]
impl CommandBus for InMemoryCommandBus {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn send<C: Command + 'static>(&self, command: C) -> Result<(), Self::Error> {
        let type_id = TypeId::of::<C>();
        let handler = self.handlers.get(&type_id).ok_or("No handler registered for command")?;

        handler.handle(Box::new(command)).await
    }
}
