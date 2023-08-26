use std::path::PathBuf;

use crate::{command::Command, event::Event};

use super::{ParseRecipe, Recipe, RecipeParsed};

pub struct Null {}

impl Recipe for Null {
    fn can_parse(&self, command: &Command<ParseRecipe>) -> bool {
        if command.payload.filepath == String::new() {
            return true;
        }
        false
    }

    fn parse(&self, _command: &Command<ParseRecipe>) -> Option<Event<RecipeParsed>> {
        Some(Event {
            event_type: 0,
            payload: RecipeParsed {
                output: PathBuf::from("."),
            },
        })
    }

    fn identifier(&self) -> String {
        String::from("null")
    }
}
