use crate::{
    Context,
    commands::CommandOutcome,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    debt::update_debts,
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translate, TranslateWithArgs},
    traveler::Traveler,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn clear_travelers(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<CommandOutcome, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

    let travelers = Traveler::db_select(db.clone(), msg.chat.id).await;
    match travelers {
        Ok(list) if !list.is_empty() => {
            // Check if any traveler has associated expenses.
            let mut travelers_with_expenses = Vec::new();
            for traveler in &list {
                match Expense::db_select_by_payer(db.clone(), traveler.clone()).await {
                    Ok(expenses) if !expenses.is_empty() => {
                        travelers_with_expenses.push(traveler.name.to_string());
                    }
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("{err}");
                        return Err(CommandError::ClearTravelers);
                    }
                }
            }

            if !travelers_with_expenses.is_empty() {
                let names = travelers_with_expenses.join(", ");
                tracing::warn!(
                    "Unable to clear travelers because some have associated expenses: {names}"
                );
                return Ok(CommandOutcome::Failure(
                    i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES.translate_with_args(
                        ctx,
                        &hashmap! { i18n::args::TRAVELERS.into() => names.into() },
                    ),
                ));
            }

            // Only delete travelers; transfers cascade via DB relationships.
            let count = list.len();
            match Traveler::db_delete_all(db.clone(), msg.chat.id).await {
                Ok(_) => {
                    if let Err(err_update) = update_debts(db, msg.chat.id).await {
                        tracing::warn!("{err_update}");
                    }
                    tracing::debug!("{LOG_DEBUG_SUCCESS}");
                    tracing::info!("{count} travelers cleared");
                    Ok(CommandOutcome::Success(
                        i18n::commands::CLEAR_TRAVELERS_OK.translate(ctx),
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::ClearTravelers)
                }
            }
        }
        Ok(_) => {
            tracing::warn!("No travelers to clear");
            Ok(CommandOutcome::Failure(
                i18n::commands::CLEAR_TRAVELERS_NOT_FOUND.translate(ctx),
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ClearTravelers)
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

    test! { clear_travelers_shows_confirmation,
        let db = db().await;
        let mut bot = TestBot::new(db, "/cleartravelers");
        let response = i18n::dialogues::CLEAR_TRAVELERS_CONFIRM.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRAVELERS_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db, "/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRAVELERS_NOT_FOUND.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_has_expenses,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_expense(&mut bot, "Dinner", 100.into(), "Alice", &["all"]).await;

        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES
            .translate_with_args_default(
                &hashmap! { i18n::args::TRAVELERS.into() => "Alice".into() },
            );
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_has_expenses_multiple,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_expense(&mut bot, "Dinner", 100.into(), "Alice", &["all"]).await;
        helpers::add_expense(&mut bot, "Lunch", 50.into(), "Bob", &["all"]).await;

        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES
            .translate_with_args_default(
                &hashmap! { i18n::args::TRAVELERS.into() => "Alice, Bob".into() },
            );
        bot.test_last_message(&response).await;
    }
}
