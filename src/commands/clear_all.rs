use crate::{
    Context,
    commands::CommandOutcome,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    debt::update_debts,
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translate},
    transfer::Transfer,
    transferred_to::TransferredTo,
    traveler::Traveler,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn clear_all(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<CommandOutcome, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

    // Check if there is anything to clear.
    let has_travelers = Traveler::db_select(db.clone(), msg.chat.id)
        .await
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let has_expenses = Expense::db_select(db.clone(), msg.chat.id)
        .await
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let has_transfers = Transfer::transfers(db.clone(), msg.chat.id)
        .await
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    if !has_travelers && !has_expenses && !has_transfers {
        tracing::warn!("Nothing to clear");
        return Ok(CommandOutcome::Failure(
            i18n::commands::CLEAR_ALL_NOT_FOUND.translate(ctx),
        ));
    }

    if let Err(err) = Expense::db_delete_all(db.clone(), msg.chat.id).await {
        tracing::error!("{err}");
        return Err(CommandError::ClearAll);
    }
    if let Err(err) = TransferredTo::db_delete_all(db.clone(), msg.chat.id).await {
        tracing::error!("{err}");
        return Err(CommandError::ClearAll);
    }
    if let Err(err) = Traveler::db_delete_all(db.clone(), msg.chat.id).await {
        tracing::error!("{err}");
        return Err(CommandError::ClearAll);
    }
    if let Err(err_update) = update_debts(db, msg.chat.id).await {
        tracing::warn!("{err_update}");
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    tracing::info!("All data cleared");
    Ok(CommandOutcome::Success(
        i18n::commands::CLEAR_ALL_OK.translate(ctx),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate},
        tests::{TestBot, helpers},
    };
    use rust_decimal::Decimal;

    test! { clear_all_shows_confirmation,
        let db = db().await;
        let mut bot = TestBot::new(db, "/clearall");
        let response = i18n::dialogues::CLEAR_ALL_CONFIRM.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_all_confirm_ok_travelers_only,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/clearall");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_ALL_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_all_confirm_ok_with_expenses_and_transfers,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_expense(&mut bot, "Dinner", 100.into(), "Alice", &["all"]).await;
        helpers::transfer(&mut bot, "Bob", "Alice", Decimal::from(50)).await;

        bot.update("/clearall");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_ALL_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_all_confirm_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db, "/clearall");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_ALL_NOT_FOUND.translate_default();
        bot.test_last_message(&response).await;
    }
}
