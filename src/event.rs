use std::fmt::Debug;

pub trait Event: Debug + Clone + Send + Sync {
    fn event_type(&self) -> &'static str;
}
