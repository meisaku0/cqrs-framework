use std::fmt::Debug;

pub trait Event: Debug + Clone + Send + Sync {
    type AggregateId: Debug + Clone + Send + Sync;
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> &Self::AggregateId;
}
