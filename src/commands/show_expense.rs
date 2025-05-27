use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    expense_details::ExpenseDetails,
    i18n::{self, Translate, translate_with_args, translate_with_args_default},
    trace_command_db,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn show_expense(
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
            // Retrieve expense details from db
            let select_res = ExpenseDetails::expense_details(db, msg.chat.id, number).await;
            match select_res {
                Ok(Some(expense_details)) => {
                    let reply = expense_details.translate(ctx.clone());
                    tracing::debug!(LOG_DEBUG_SUCCESS);
                    Ok(reply)
                }
                Ok(_) => {
                    tracing::warn!(
                        "{}",
                        translate_with_args_default(
                            i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                            &hashmap! {i18n::args::NUMBER.into() => number.into()},
                        )
                    );
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                        &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::ShowExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowExpense { number })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        db::db,
        expense_details::ExpenseDetails,
        i18n::{self, Translate, translate_with_args_default},
        tests::{TestBot, helpers},
    };
    use maplit::hashmap;
    use rust_decimal::Decimal;

    test! { show_expense_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob", "Charlie" and "David"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;
        helpers::add_traveler(&mut bot, "David").await;

        // Add expense #1
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            Decimal::from_str("100.8").unwrap(),
            "Alice",
            &["all"]
        ).await;

        // Add expense #2
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Alice",
            &["Alice:100", "end"]
        ).await;

        // Add expense #3
        helpers::add_expense(
            &mut bot,
            "Test expense 3",
            100.into(),
            "Alice",
            &["Alice: 40; Bob: 40%; Charlie; David", "end"]
        ).await;

        // Add expense #4
        helpers::add_expense(
            &mut bot,
            "Test expense 4",
            100.into(),
            "Alice",
            &["Alice: 50; Bob: 50", "end"]
        ).await;

        // Add expense #5
        helpers::add_expense(
            &mut bot,
            "Test expense 5",
            100.into(),
            "Alice",
            &["Alice: 50", "all"]
        ).await;

        // Add expense #6
        helpers::add_expense(
            &mut bot,
            "Test expense 6",
            100.into(),
            "Alice",
            &["Alice: 67%; Bob: 34%", "end"]
        ).await;

        // Add expense #7
        helpers::add_expense(
            &mut bot,
            "Test expense 7",
            100.into(),
            "Alice",
            &["Alice: 110", "end", "Alice:100", "end"]
        ).await;

        for i in 1..=7 {
            // Show expense #i
            bot.update(&format!("/showexpense {i}"));
            let expense_details = ExpenseDetails::expense_details(db.clone(), bot.chat_id(), i).await.unwrap().unwrap();
            let response = expense_details.translate_default();
            bot.test_last_message(&response).await;
        };
    }

    test! { show_expense_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showexpense 1");
        let response = translate_with_args_default(
            i18n::commands::SHOW_EXPENSE_NOT_FOUND,
            &hashmap! {i18n::args::NUMBER.into() => 1.into()},
        );
        bot.test_last_message(&response).await;
    }
}
