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

/// Returns a list of `(traveler, expenses)` pairs for every traveler
/// in `chat_id` that has at least one associated expense.
pub async fn travelers_with_expenses(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
) -> Result<Vec<(Traveler, Vec<Expense>)>, CommandError> {
    let travelers = Traveler::db_select(db.clone(), chat_id)
        .await
        .map_err(|err| {
            tracing::error!("{err}");
            CommandError::ClearTravelers
        })?;

    let mut result = Vec::new();
    for traveler in &travelers {
        let expenses = Expense::db_select_by_payer(db.clone(), traveler.clone())
            .await
            .map_err(|err| {
                tracing::error!("{err}");
                CommandError::ClearTravelers
            })?;
        if !expenses.is_empty() {
            result.push((traveler.clone(), expenses));
        }
    }
    Ok(result)
}

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
            let with_expenses = travelers_with_expenses(db.clone(), msg.chat.id).await?;

            if !with_expenses.is_empty() {
                let names = with_expenses
                    .iter()
                    .map(|(t, _)| t.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
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
        dialogues::pending_command_dialogue::clear_travelers::{SHOW_ALL_CALLBACK, SHOW_PREFIX},
        expense::Expense,
        i18n::{self, Translate, TranslateWithArgs},
        tests::{TestBot, helpers},
        traveler::Traveler,
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
        let has_expenses = i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES
            .translate_with_args_default(
                &hashmap! { i18n::args::TRAVELERS.into() => "Alice".into() },
            );
        // Single traveler with expenses — expenses shown directly (no keyboard).
        let alice = Traveler::db_select_by_name(db.clone(), bot.chat_id(), &"Alice".parse().unwrap())
            .await
            .unwrap()
            .unwrap();
        let expenses = Expense::db_select_by_payer(db, alice).await.unwrap();
        let expenses_reply: String = expenses
            .iter()
            .map(|e| e.translate(bot.context()))
            .collect::<Vec<_>>()
            .join("\n");
        let response = format!("{has_expenses}\n\n{expenses_reply}");
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
        let has_expenses = i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES
            .translate_with_args_default(
                &hashmap! { i18n::args::TRAVELERS.into() => "Alice, Bob".into() },
            );
        let prompt = i18n::dialogues::CLEAR_TRAVELERS_SHOW_EXPENSES_PROMPT.translate_default();
        let response = format!("{has_expenses}\n\n{prompt}");
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_no,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("no");
        let response = helpers::cancel_ok_for(i18n::commands::RUNNING_PROCESS_CLEAR_TRAVELERS);
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_unknown_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "/cleartravelers");
        bot.dispatch().await;
        // Send unrecognized text — should re-prompt.
        bot.update("maybe");
        let response = i18n::dialogues::CLEAR_TRAVELERS_CONFIRM.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_cancel_during_confirm,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("/cancel");
        let response = helpers::cancel_ok_for(i18n::commands::RUNNING_PROCESS_CLEAR_TRAVELERS);
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_cancel_during_show_expenses,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_expense(&mut bot, "Dinner", 100.into(), "Alice", &["all"]).await;
        helpers::add_expense(&mut bot, "Lunch", 50.into(), "Bob", &["all"]).await;

        // Reach ShowExpenses state (needs multiple travelers with expenses).
        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        bot.dispatch().await;

        // Cancel from ShowExpenses state.
        bot.update("/cancel");
        let response = helpers::cancel_ok_for(i18n::commands::RUNNING_PROCESS_CLEAR_TRAVELERS);
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_confirm_yes_no_expenses_deletes,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Travelers without expenses — confirm yes should delete directly.
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        let response = i18n::commands::CLEAR_TRAVELERS_OK.translate_default();
        bot.test_last_message(&response).await;

        // Verify travelers are gone.
        bot.update("/listtravelers");
        let response = i18n::commands::LIST_TRAVELERS_NOT_FOUND.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { clear_travelers_show_expenses_single_traveler,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;
        helpers::add_expense(&mut bot, "Dinner", 100.into(), "Alice", &["all"]).await;
        helpers::add_expense(&mut bot, "Lunch", 50.into(), "Bob", &["all"]).await;

        // Multiple travelers with expenses — keyboard is shown.
        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        bot.dispatch().await;

        // Select Alice via callback. Use her DB number field.
        let alice = Traveler::db_select_by_name(db.clone(), bot.chat_id(), &"Alice".parse().unwrap())
            .await
            .unwrap()
            .unwrap();
        bot.update_callback(&format!("{SHOW_PREFIX}{}", alice.number));

        // Build expected response from DB expenses.
        let expenses = Expense::db_select_by_payer(db, alice).await.unwrap();
        let expected: String = expenses
            .iter()
            .map(|e| e.translate(bot.context()))
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&expected).await;
    }

    test! { clear_travelers_show_expenses_all,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_expense(&mut bot, "Dinner", 100.into(), "Alice", &["all"]).await;
        helpers::add_expense(&mut bot, "Lunch", 50.into(), "Bob", &["all"]).await;

        // Reach ShowExpenses state.
        bot.update("/cleartravelers");
        bot.dispatch().await;
        bot.update("yes");
        bot.dispatch().await;

        // Select "All" via callback.
        bot.update_callback(SHOW_ALL_CALLBACK);

        // Build expected response: all expenses from travelers that have them.
        let travelers = Traveler::db_select(db.clone(), bot.chat_id()).await.unwrap();
        let mut all_expenses = Vec::new();
        for t in &travelers {
            let exps = Expense::db_select_by_payer(db.clone(), t.clone()).await.unwrap();
            all_expenses.extend(exps);
        }
        let expected: String = all_expenses
            .iter()
            .map(|e| e.translate(bot.context()))
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&expected).await;
    }
}
