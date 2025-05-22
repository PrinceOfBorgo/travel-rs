use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, Translate, translate, translate_with_args},
    trace_command,
    transfer::Transfer,
    traveler::Name,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_transfers(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);

    let list_res = if name.is_empty() {
        Transfer::transfers(db, msg.chat.id).await
    } else {
        Transfer::transfers_by_name(db, msg.chat.id, name.clone()).await
    };

    match list_res {
        Ok(transfers) => {
            let reply = if transfers.is_empty() {
                if name.is_empty() {
                    translate(ctx, i18n::commands::LIST_TRANSFERS_NOT_FOUND)
                } else {
                    translate_with_args(
                        ctx,
                        i18n::commands::LIST_TRANSFERS_NAME_NOT_FOUND,
                        &hashmap! {i18n::args::NAME.into() => name.into()},
                    )
                }
            } else {
                transfers
                    .into_iter()
                    .map(|transfer| transfer.translate(ctx.clone()))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(LOG_DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListTransfers { name })
        }
    }
}
