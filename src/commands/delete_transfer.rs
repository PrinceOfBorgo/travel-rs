use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, translate_with_args, translate_with_args_default},
    trace_command,
    transferred_to::TransferredTo,
    utils::update_debts,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_transfer(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);

    // Check if transfer relation exists on db
    let count_res = TransferredTo::db_count(db.clone(), msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete transfer relation from db
            let delete_res = TransferredTo::db_delete(db.clone(), msg.chat.id, number).await;
            match delete_res {
                Ok(_) => {
                    if let Err(err_update) = update_debts(db, msg.chat.id).await {
                        tracing::warn!("{err_update}");
                    }
                    tracing::debug!(LOG_DEBUG_SUCCESS);
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::DELETE_TRANSFER_OK,
                        &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteTransfer { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::DELETE_TRANSFER_NOT_FOUND,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::DELETE_TRANSFER_NOT_FOUND,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteTransfer { number })
        }
    }
}
