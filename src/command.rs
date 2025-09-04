use crate::{Aggregate, Event};

pub trait Command: Send + Send {
    type Aggregate: Aggregate;
    type Event: Event;
    type Error;
    fn aggregate_id(&self) -> &<Self::Event as Event>::AggregateId;
    fn execute(&self, aggregate: &Self::Aggregate) -> Result<Vec<&Self::Event>, Self::Error>;
}
