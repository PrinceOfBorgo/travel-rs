use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::{self, translate_with_args, translate_with_args_default},
    trace_command,
    utils::update_debts,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_expense(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);

    // Check if expense exists on db
    let count_res = Expense::db_count_by_number(db.clone(), msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete expense from db
            let delete_res = Expense::db_delete_by_number(db.clone(), msg.chat.id, number).await;
            match delete_res {
                Ok(_) => {
                    if let Err(err_update) = update_debts(db, msg.chat.id).await {
                        tracing::warn!("{err_update}");
                    }
                    tracing::debug!(LOG_DEBUG_SUCCESS);
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::DELETE_EXPENSE_OK,
                        &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::DELETE_EXPENSE_NOT_FOUND,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::DELETE_EXPENSE_NOT_FOUND,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteExpense { number })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, translate_default, translate_with_args_default},
        tests::TestBot,
    };
    use maplit::hashmap;

    test! { delete_expense_ok,
        let db = db().await;

        // Add traveler "Alice"
        let mut bot = TestBot::new(db.clone(), "/addtraveler Alice");
        bot.dispatch().await;

        // Add expense
        bot.update("/addexpense");
        bot.dispatch().await;
        // 1. Set description
        bot.update("Test expense");
        bot.dispatch().await;
        // 2. Set amount
        bot.update("100");
        bot.dispatch().await;
        // 3. Set payer
        bot.update("Alice");
        bot.dispatch().await;
        // 4. Split expense
        bot.update("all");
        bot.dispatch().await;

        // Delete expense #1
        bot.update("/deleteexpense 1");
        let response = translate_with_args_default(
            i18n::commands::DELETE_EXPENSE_OK,
            &hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_expense_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deleteexpense 1");
        let response = translate_with_args_default(
            i18n::commands::DELETE_EXPENSE_NOT_FOUND,
            &hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_expense_twice,
        let db = db().await;

        // Add traveler "Alice"
        let mut bot = TestBot::new(db.clone(), "/addtraveler Alice");
        bot.dispatch().await;

        // Add expense
        bot.update("/addexpense");
        bot.dispatch().await;
        // 1. Set description
        bot.update("Test expense");
        bot.dispatch().await;
        // 2. Set amount
        bot.update("100");
        bot.dispatch().await;
        // 3. Set payer
        bot.update("Alice");
        bot.dispatch().await;
        // 4. Split expense
        bot.update("all");
        bot.dispatch().await;

        // Delete expense #1 -> ok
        bot.update("/deleteexpense 1");
        let response = translate_with_args_default(
            i18n::commands::DELETE_EXPENSE_OK,
            &hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;

        // Delete expense #1 again -> not found
        let response = translate_with_args_default(
            i18n::commands::DELETE_EXPENSE_NOT_FOUND,
            &hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_expense_invalid_usage,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deleteexpense");
        let help_message = translate_default(i18n::help::HELP_DELETE_EXPENSE);
        let err = translate_with_args_default(
            i18n::commands::INVALID_COMMAND_USAGE,
            &hashmap! {
                i18n::args::COMMAND.into() => "/deleteexpense".into(),
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
