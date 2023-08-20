use std::rc::Rc;

use taqueria::{command, command::CommandHandler, config, event::EventHandler, notifier, recipe};

fn main() {
    env_logger::init();
    let conf =
        config::Config::read(config::Config::path()).expect("Could not initialize configuration.");
    let notifier = Rc::new(notifier::console::ConsoleNotifier {});
    let carne_asade = Box::new(recipe::carne_asade::CarneAsada {});
    let mut parse_recipe_command_handler = recipe::ParseRecipeCommandHandler::default();
    let recipe_parsed_event_handler = recipe::RecipeParsedEventHandler { notifier };
    parse_recipe_command_handler.register(carne_asade);
    let cmd = command::Command::<recipe::ParseRecipe> {
        command_type: 0,
        payload: recipe::ParseRecipe {
            basepath: conf.basepath,
            filepath: conf.filepath,
            identifier: uuid::Uuid::new_v4(),
        },
    };
    let evt = parse_recipe_command_handler
        .handle(&cmd)
        .expect("Could not handle recipe.");
    recipe_parsed_event_handler.handle(evt);
}
