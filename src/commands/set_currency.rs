use crate::{
    Context,
    chat::Chat,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, translate_with_args},
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn set_currency(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    currency: &str,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    // Update chat currency on db
    let update_res = Chat::db_update_currency(db, msg.chat.id, currency).await;
    match update_res {
        Ok(_) => {
            tracing::debug!(DEBUG_SUCCESS);
            {
                let mut ctx_guard = ctx.lock().expect("Failed to lock context");
                ctx_guard.currency = currency.to_owned();
            }

            Ok(translate_with_args(
                ctx.clone(),
                i18n::commands::SET_CURRENCY_OK,
                &hashmap! {i18n::args::CURRENCY.into() => currency.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::SetCurrency {
                currency: currency.to_owned(),
            })
        }
    }
}
