pub mod redis;

use serde::{Deserialize, Serialize};

use crate::command::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceiveError;

pub trait Broker {
    fn receive(&mut self) -> Option<Command<serde_json::Value>>;
}
