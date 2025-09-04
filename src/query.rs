use std::fmt::Debug;

pub trait Query: Debug + Clone + Send + Sync {
    type Result: Send + Sync;
}
