use crate::{
    Context,
    commands::CommandOutcome,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, TranslateWithArgs},
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn add_traveler(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<CommandOutcome, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

    // Check if traveler exists on db
    let count_res = Traveler::db_count(db.clone(), msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            tracing::warn!(
                "{}",
                i18n::commands::ADD_TRAVELER_ALREADY_ADDED.translate_with_args_default(
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                )
            );
            Ok(CommandOutcome::Failure(
                i18n::commands::ADD_TRAVELER_ALREADY_ADDED
                    .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => name.into()}),
            ))
        }
        Ok(_) => {
            // Create traveler on db
            let create_res = Traveler::db_create(db, msg.chat.id, &name).await;
            match create_res {
                Ok(_) => {
                    tracing::debug!("{LOG_DEBUG_SUCCESS}");
                    Ok(CommandOutcome::Success(
                        i18n::commands::ADD_TRAVELER_OK.translate_with_args(
                            ctx,
                            &hashmap! {i18n::args::NAME.into() => name.into()},
                        ),
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::AddTraveler { name })
                }
            }
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::AddTraveler {
                name: name.to_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate, TranslateWithArgs},
        tests::TestBot,
    };
    use maplit::hashmap;

    test! { add_traveler_ok,
        let db = db().await;

        let mut bot = TestBot::new(db, "/addtraveler Alice");
        let response = i18n::commands::ADD_TRAVELER_OK.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { add_traveler_already_added,
        let db = db().await;

        // Add traveler "Alice"
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        // Try to add traveler "Alice" again
        let response = i18n::commands::ADD_TRAVELER_ALREADY_ADDED.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { add_traveler_already_added_case_insensitive,
        let db = db().await;

        // Add traveler "Alice"
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        // Try to add traveler "alice" — should be rejected as a duplicate
        // because traveler-name uniqueness is case-insensitive within a chat.
        bot.update("/addtraveler alice");
        let response = i18n::commands::ADD_TRAVELER_ALREADY_ADDED.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { add_traveler_empty_input,
        let db = db().await;

        // Add traveler without specifying a name -> ask for name
        let mut bot = TestBot::new(db, "/addtraveler");
        let response = i18n::dialogues::ADD_TRAVELER_ASK_NAME.translate_default();
        bot.test_last_message(&response).await;

        // Provide the name as a follow-up message
        bot.update("Alice");
        let response = i18n::commands::ADD_TRAVELER_OK.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    // The inline form `/addtraveler <name>` is a regular command, not a
    // dialogue, so it must run even while another dialogue is active and
    // must not disturb that dialogue
    test! { inline_add_traveler_runs_during_add_expense_dialogue,
        let db = db().await;

        // Start AddExpense dialogue
        let mut bot = TestBot::new(db, "/addexpense");
        bot.dispatch().await;

        // Inline `/addtraveler Alice` falls through to the command handler
        bot.update("/addtraveler Alice");
        let response = i18n::commands::ADD_TRAVELER_OK.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;

        // AddExpense is still alive: feeding text advances it to the next prompt
        bot.update("My expense");
        let response = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        bot.test_last_message(&response).await;
    }
}
