pub mod console;

pub trait Notifier {
    fn success(&self, msg: String);
    fn failure(&self, msg: String);
}
