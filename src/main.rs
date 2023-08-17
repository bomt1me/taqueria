use std::rc::Rc;

use taqueria::{
    broker, command, command::CommandHandler, config, event::EventHandler, notifier, recipe,
};

fn main() {
    env_logger::init();
    let conf =
        config::Config::read(config::Config::path()).expect("Could not initialize configuration.");
    let mut redis =
        Rc::new(broker::redis::Redis::new(conf).expect("Could not initialize redis client"));
    let notifier = Rc::new(notifier::console::ConsoleNotifier {});
    let null_parser = Box::new(recipe::null::NullParser {});
    let mut parse_recipe_command_handler = recipe::ParseRecipeCommandHandler::default();
    let recipe_parsed_event_handler = recipe::RecipeParsedEventHandler { notifier };
    parse_recipe_command_handler.register(null_parser);
    Rc::get_mut(&mut redis)
        .and_then(broker::Broker::receive)
        .map(|m| command::Command::<recipe::ParseRecipe> {
            command_type: m.command_type,
            payload: serde_json::from_value(m.payload).expect("could not deserialize payload"),
        })
        .and_then(|m| parse_recipe_command_handler.handle(&m))
        .map(|e| {
            recipe_parsed_event_handler.handle(e);
            Some(())
        });
}
