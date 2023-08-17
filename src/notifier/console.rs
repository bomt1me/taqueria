use log::{error, info};

use super::Notifier;

pub struct ConsoleNotifier;

impl Notifier for ConsoleNotifier {
    fn success(&self, msg: String) {
        info!("Success: {}", msg);
    }
    fn failure(&self, msg: String) {
        error!("Failure: {}", msg);
    }
}
