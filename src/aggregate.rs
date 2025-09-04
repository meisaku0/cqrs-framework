use crate::Event;

pub trait Aggregate: Default + Clone + Send + Sync {
    type Event: Event;
    fn apply(&mut self, event: Self::Event);
    fn version(&self) -> u64;
    fn increment_version(&mut self);
}
