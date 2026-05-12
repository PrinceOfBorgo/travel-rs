use crate::{
    Context,
    commands::CommandOutcome,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    debt::update_debts,
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translate},
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn clear_expenses(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<CommandOutcome, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

    let expenses = Expense::db_select(db.clone(), msg.chat.id).await;
    match expenses {
        Ok(list) if !list.is_empty() => {
            match Expense::db_delete_all(db.clone(), msg.chat.id).await {
                Ok(_) => {
                    if let Err(err_update) = update_debts(db, msg.chat.id).await {
                        tracing::warn!("{err_update}");
                    }
                    tracing::debug!("{LOG_DEBUG_SUCCESS}");
                    tracing::info!("{} expenses cleared", list.len());
                    Ok(CommandOutcome::Success(
                        i18n::commands::CLEAR_EXPENSES_OK.translate(ctx),
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::ClearExpenses)
                }
            }
        }
        Ok(_) => {
            tracing::warn!("No expenses to clear");
            Ok(CommandOutcome::Failure(
                i18n::commands::CLEAR_EXPENSES_NOT_FOUND.translate(ctx),
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ClearExpenses)
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

    test! { clear_expenses_shows_confirmation,
        let db = db().await;
        let mut bot = TestBot::new(db, "/clearexpenses");
        let response = i18n::dialogues::CLEAR_EXPENSES_CONFIRM.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_expenses_confirm_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_expense(&mut bot, "Test expense", 100.into(), "Alice", &["all"]).await;

        bot.update("/clearexpenses");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_EXPENSES_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_expenses_confirm_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db, "/clearexpenses");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_EXPENSES_NOT_FOUND.translate_default();
        bot.test_last_message(&response).await;
    }
}
