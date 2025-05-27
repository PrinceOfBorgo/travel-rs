use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translate, translate, translate_with_args},
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_expenses(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    description: &str,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);

    let list_res = if description.is_empty() {
        Expense::db_select(db, msg.chat.id).await
    } else {
        Expense::db_select_by_descr(db, msg.chat.id, description.to_owned()).await
    };

    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                if description.is_empty() {
                    translate(ctx, i18n::commands::LIST_EXPENSES_NOT_FOUND)
                } else {
                    translate_with_args(
                        ctx,
                        i18n::commands::LIST_EXPENSES_DESCR_NOT_FOUND,
                        &hashmap! {i18n::args::DESCRIPTION.into() => description.into()},
                    )
                }
            } else {
                expenses
                    .into_iter()
                    .map(|expense| expense.translate(ctx.clone()))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(LOG_DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListExpenses {
                description: description.to_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        expense::Expense,
        i18n::{self, Translate, translate_default, translate_with_args_default},
        tests::{TestBot, helpers},
    };
    use maplit::hashmap;

    test! { list_expenses_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice" and "Bob"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Add first expense
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            100.into(),
            "Alice",
            &["all"],
        ).await;
        // Add another expense
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Bob",
            &["Alice:70;Bob", "end"],
        ).await;

        // List expenses
        bot.update("/listexpenses");
        let expenses = Expense::db_select(db, bot.chat_id()).await.unwrap();
        // Check that two expenses has been recorded
        assert_eq!(expenses.len(), 2);
        let response = expenses
            .into_iter()
            .map(|expense| expense.translate_default())
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;
    }

    test! { list_expenses_by_descr_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice" and "Bob"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Add first expense
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            100.into(),
            "Alice",
            &["all"],
        ).await;
        // Add another expense
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Bob",
            &["Alice:70;Bob", "end"],
        ).await;

        // List expenses with description matching "1"
        bot.update("/listexpenses 1");
        let expenses = Expense::db_select_by_descr(db, bot.chat_id(), String::from("1")).await.unwrap();
        // Check that only one expense is returned
        assert_eq!(expenses.len(), 1);
        let response = expenses
            .into_iter()
            .map(|expense| expense.translate_default())
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;
    }

    test! { list_expenses_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/listexpenses");
        let response = translate_default(
            i18n::commands::LIST_EXPENSES_NOT_FOUND,
        );
        bot.test_last_message(&response).await;
    }

    test! { list_expenses_descr_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice" and "Bob"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Add one expense
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            100.into(),
            "Alice",
            &["all"],
        ).await;

        // List expenses with description matching "2" -> not found
        bot.update("/listexpenses 2");
        let response = translate_with_args_default(
            i18n::commands::LIST_EXPENSES_DESCR_NOT_FOUND,
            &hashmap! {i18n::args::DESCRIPTION.into() => "2".into()},
        );
        bot.test_last_message(&response).await;
    }
}
