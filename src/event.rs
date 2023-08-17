use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event<T> {
    pub event_type: u64,
    pub payload: T,
}

pub trait EventHandler<T> {
    fn handle(&self, event: Event<T>);
}
