use async_trait::async_trait;

use crate::Aggregate;

pub trait Snapshot: Clone + Send + Sync {
    fn version(&self) -> u64;
}

#[async_trait]
pub trait SnapshotStore<S: Snapshot, Id> {
    type Error;

    async fn save_snapshot(&self, aggregate_id: &Id, snapshot: S) -> Result<(), Self::Error>;
    async fn get_snapshot(&self, aggregate_id: &Id) -> Result<Option<S>, Self::Error>;
}

impl<A: Aggregate + Clone> Snapshot for A {
    fn version(&self) -> u64 { self.version() }
}
