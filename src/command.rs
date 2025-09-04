use crate::Aggregate;

pub trait Command: Send + Sync {
    type Aggregate: Aggregate;
    type Error;
    type AggregateId;

    fn aggregate_id(&self) -> &Self::AggregateId;

    fn execute(&self, aggregate: &Self::Aggregate) -> Result<Vec<<Self::Aggregate as Aggregate>::Event>, Self::Error>;
}
