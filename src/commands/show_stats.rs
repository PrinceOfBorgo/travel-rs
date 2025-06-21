use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::Translate,
    stats::Stats,
    trace_command_db,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn show_stats(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");
    let stats_res = Stats::stats(db, msg.chat.id).await;
    match stats_res {
        Ok(Some(stats)) => {
            let reply = stats.translate(ctx);
            tracing::debug!("{LOG_DEBUG_SUCCESS}");
            Ok(reply)
        }
        Ok(_) => {
            tracing::warn!("Couldn't retrieve stats, no data found.");
            Err(CommandError::ShowStats)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowStats)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::Translate,
        stats::Stats,
        tests::{TestBot, helpers},
    };
    use rust_decimal::Decimal;
    use std::str::FromStr;

    test! { show_stats_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;

        // Test expense split evenly
        helpers::add_expense(&mut bot, "Even split dinner", 90.into(), "Alice", &["all"]).await;

        // Test expense with specific amounts
        helpers::add_expense(
            &mut bot,
            "Custom split lunch",
            100.into(),
            "Bob",
            &["Alice:40; Bob:30; Charlie:30", "end"],
        )
        .await;

        // Test expense with percentages
        helpers::add_expense(
            &mut bot,
            "Percentage split breakfast",
            50.into(),
            "Charlie",
            &["Alice:50%; Bob:25%; Charlie:25%", "end"],
        )
        .await;

        // Have Bob make a transfer to Alice
        helpers::transfer(
            &mut bot,
            "Bob",
            "Alice",
            Decimal::from_str("50.17").unwrap(),
        )
        .await;

        // Show stats
        bot.update("/showstats");
        let response = Stats::stats(db, bot.chat_id()).await.unwrap().unwrap().translate_default();
        bot.test_last_message(&response).await;
    }

    test! { show_stats_ok_empty,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showstats");
        let response = Stats::default().translate_default();
        bot.test_last_message(&response).await;
    }
}
