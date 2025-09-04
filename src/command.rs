use crate::{Aggregate, Event};

pub trait Command: Send + Sync {
    type Aggregate: Aggregate;
    type AggregateId;
    type Error;
    type Event: Event;

    fn aggregate_id(&self) -> &Self::AggregateId;

    fn execute(&self, aggregate: &Self::Aggregate) -> Result<Vec<Self::Event>, Self::Error>;
}
