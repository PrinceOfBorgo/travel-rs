use crate::Context;
use std::sync::{Arc, Mutex};

pub trait Translate {
    fn translate(&self, ctx: Arc<Mutex<Context>>) -> String;
    fn translate_default(&self) -> String {
        self.translate(Arc::new(Mutex::new(Context::default())))
    }
}
