use crate::{command::Command, event::Event};

use super::{ParseRecipe, Recipe, RecipeParsed};

pub struct NullParser {}

impl Recipe for NullParser {
    fn parse(&self, command: &Command<ParseRecipe>) -> Option<Event<RecipeParsed>> {
        Some(Event {
            event_type: 0,
            payload: RecipeParsed {
                name: format!("{}:{}", String::from("null"), command.payload.filepath),
                guacamole: Vec::new(),
                beans: Vec::new(),
            },
        })
    }

    fn identifier(&self) -> String {
        String::from("null")
    }
}
