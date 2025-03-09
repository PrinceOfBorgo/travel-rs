use crate::Context;
use std::sync::{Arc, Mutex};

pub trait HelpMessage {
    fn help_message(&self, ctx: Arc<Mutex<Context>>) -> String;
}
