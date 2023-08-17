use serde::{Deserialize, Serialize};

use crate::event::Event;

#[derive(Serialize, Deserialize, Debug)]
pub struct Command<T> {
    pub command_type: u64,
    pub payload: T,
}

impl TryFrom<String> for Command<serde_json::Value> {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}

pub trait CommandHandler<T, R> {
    fn handle(&self, command: &Command<T>) -> Option<Event<R>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_good_msg_when_from_msg_then_command() {
        let json: &str = r#"
            {
              "command_type": 10,
              "payload": ""
            }
            "#;
        let command = Command::try_from(String::from(json)).expect("Parsing failed");
        assert_eq!(command.command_type, 10);
    }
}
