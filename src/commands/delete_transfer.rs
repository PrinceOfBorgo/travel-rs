use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, TranslateWithArgs},
    trace_command_db,
    transferred_to::TransferredTo,
    utils::update_debts,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn delete_transfer(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

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
                    tracing::debug!("{LOG_DEBUG_SUCCESS}");
                    Ok(i18n::commands::DELETE_TRANSFER_OK.translate_with_args(
                        ctx,
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
                i18n::commands::DELETE_TRANSFER_NOT_FOUND.translate_with_args_default(
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                )
            );
            Ok(i18n::commands::DELETE_TRANSFER_NOT_FOUND
                .translate_with_args(ctx, &hashmap! {i18n::args::NUMBER.into() => number.into()}))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteTransfer { number })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate, TranslateWithArgs},
        tests::{TestBot, helpers},
    };
    use maplit::hashmap;

    test! { delete_transfer_ok,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        // Transfer 100 from Alice to Bob
        helpers::add_travelers_and_transfer(&mut bot, "Alice", "Bob", 100.into()).await;

        // Delete transfer #1
        bot.update("/deletetransfer 1");
        let response = i18n::commands::DELETE_TRANSFER_OK.translate_with_args_default(&hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_transfer_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer 1");
        let response = i18n::commands::DELETE_TRANSFER_NOT_FOUND.translate_with_args_default(&hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_transfer_twice,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        // Transfer 100 from Alice to Bob
        helpers::add_travelers_and_transfer(&mut bot, "Alice", "Bob", 100.into()).await;

        // Delete transfer #1 -> ok
        bot.update("/deletetransfer 1");
        let response = i18n::commands::DELETE_TRANSFER_OK.translate_with_args_default(&hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;

        // Delete transfer #1 again -> not found
        let response = i18n::commands::DELETE_TRANSFER_NOT_FOUND.translate_with_args_default(&hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_transfer_invalid_usage,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer");
        let help_message = i18n::help::HELP_DELETE_TRANSFER.translate_default();
        let err = i18n::commands::INVALID_COMMAND_USAGE.translate_with_args_default(&hashmap! {
                i18n::args::COMMAND.into() => "/deletetransfer".into(),
                i18n::args::HELP_MESSAGE.into() => help_message.into()
            },
        );
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );
    }
}
