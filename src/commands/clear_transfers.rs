use crate::{
    Context,
    commands::CommandOutcome,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    debt::update_debts,
    errors::CommandError,
    i18n::{self, Translate},
    transfer::Transfer,
    transferred_to::TransferredTo,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn clear_transfers(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<CommandOutcome, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

    let transfers = Transfer::transfers(db.clone(), msg.chat.id).await;
    match transfers {
        Ok(list) if !list.is_empty() => {
            match TransferredTo::db_delete_all(db.clone(), msg.chat.id).await {
                Ok(_) => {
                    if let Err(err_update) = update_debts(db, msg.chat.id).await {
                        tracing::warn!("{err_update}");
                    }
                    tracing::debug!("{LOG_DEBUG_SUCCESS}");
                    tracing::info!("{} transfers cleared", list.len());
                    Ok(CommandOutcome::Success(
                        i18n::commands::CLEAR_TRANSFERS_OK.translate(ctx),
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::ClearTransfers)
                }
            }
        }
        Ok(_) => {
            tracing::warn!("No transfers to clear");
            Ok(CommandOutcome::Failure(
                i18n::commands::CLEAR_TRANSFERS_NOT_FOUND.translate(ctx),
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ClearTransfers)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate},
        tests::{TestBot, helpers},
    };
    use rust_decimal::Decimal;

    test! { clear_transfers_shows_confirmation,
        let db = db().await;
        let mut bot = TestBot::new(db, "/cleartransfers");
        let response = i18n::dialogues::CLEAR_TRANSFERS_CONFIRM.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_transfers_confirm_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_travelers_and_transfer(&mut bot, "Alice", "Bob", Decimal::from(50)).await;

        bot.update("/cleartransfers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRANSFERS_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_transfers_confirm_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db, "/cleartransfers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRANSFERS_NOT_FOUND.translate_default();
        bot.test_last_message(&response).await;
    }
}
