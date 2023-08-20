pub mod carne_asade;
pub mod null;

use std::path::PathBuf;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::command::{self, Command, CommandHandler};
use crate::event::{Event, EventHandler};

#[derive(Serialize, Deserialize, Debug)]
pub struct RecipeParsed {
    output: PathBuf,
}

#[derive(Debug)]
pub struct ParseRecipe {
    pub basepath: String,
    pub filepath: String,
    pub identifier: uuid::Uuid,
}

pub trait Recipe {
    fn parse(&self, command: &Command<ParseRecipe>) -> Option<Event<RecipeParsed>>;
    fn identifier(&self) -> String;
}

#[derive(Default)]
pub struct ParseRecipeCommandHandler {
    parsers: Vec<Box<dyn Recipe>>,
}

impl ParseRecipeCommandHandler {
    pub fn register(&mut self, parser: Box<dyn Recipe>) {
        if !self
            .parsers
            .iter()
            .any(|p| p.identifier() == parser.identifier())
        {
            self.parsers.push(parser);
        }
    }
}

impl CommandHandler<ParseRecipe, RecipeParsed> for ParseRecipeCommandHandler {
    fn handle(&self, command: &command::Command<ParseRecipe>) -> Option<Event<RecipeParsed>> {
        for parser in &self.parsers {
            let result = parser.parse(command);
            if result.is_some() {
                return result;
            }
        }
        None
    }
}

pub struct RecipeParsedEventHandler {
    pub notifier: Rc<dyn crate::notifier::Notifier>,
}

impl EventHandler<RecipeParsed> for RecipeParsedEventHandler {
    fn handle(&self, event: Event<RecipeParsed>) {
        self.notifier.success(String::from(
            event
                .payload
                .output
                .as_os_str()
                .to_os_string()
                .to_str()
                .expect("Could not get path."),
        ));
    }
}
