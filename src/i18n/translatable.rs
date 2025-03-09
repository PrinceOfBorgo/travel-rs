use crate::Context;
use std::sync::{Arc, Mutex};

pub trait Translatable {
    fn translate(&self, ctx: Arc<Mutex<Context>>) -> String;
    fn translate_default(&self) -> String;
}
