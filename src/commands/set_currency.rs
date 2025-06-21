use crate::{
    Context,
    chat::Chat,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, TranslateWithArgs},
    trace_command_db,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn set_currency(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    currency: &str,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");
    let currency = currency.to_uppercase();
    // Update chat currency on db
    let update_res = Chat::db_update_currency(db, msg.chat.id, &currency).await;
    match update_res {
        Ok(_) => {
            tracing::debug!("{LOG_DEBUG_SUCCESS}");
            {
                let mut ctx_guard = ctx.lock().expect("Failed to lock context");
                ctx_guard.currency = currency.to_owned();
            }

            Ok(i18n::commands::SET_CURRENCY_OK.translate_with_args(
                ctx.clone(),
                &hashmap! {i18n::args::CURRENCY.into() => currency.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::SetCurrency {
                currency: currency.to_owned(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        expense::Expense,
        i18n::{self, Translate, TranslateWithArgs},
        tests::TestBot,
        traveler::{Name, Traveler},
    };
    use maplit::hashmap;
    use std::str::FromStr;

    test! { set_currency_ok,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setcurrency TEST");
        let response = i18n::commands::SET_CURRENCY_OK.translate_with_args_default(&hashmap! {i18n::args::CURRENCY.into() => "TEST".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { set_currency_format_ok,
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

        // Retrieve traveler "Alice" and their expenses
        let traveler =
            Traveler::db_select_by_name(db.clone(), bot.chat_id(), &Name::from_str("Alice").unwrap())
                .await
                .unwrap()
                .unwrap();
        let expenses = Expense::db_select_by_payer(db, traveler).await.unwrap();
        let expense = expenses.first().unwrap();

        // Set TEST currency
        bot.update("/setcurrency TEST");
        bot.dispatch().await;

        // Check output
        bot.update("/listexpenses");
        let response = expense.translate(bot.context());
        bot.test_last_message(&response).await;

        // Set USD currency
        bot.update("/setcurrency USD");
        bot.dispatch().await;

        // Check output
        bot.update("/listexpenses");
        let response = expense.translate(bot.context());
        bot.test_last_message(&response).await;

        // Set BTC currency
        bot.update("/setcurrency BTC");
        bot.dispatch().await;

        // Check output
        bot.update("/listexpenses");
        let response = expense.translate(bot.context());
        bot.test_last_message(&response).await;
    }
}
