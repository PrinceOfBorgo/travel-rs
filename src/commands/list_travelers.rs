use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n,
    i18n::translate,
    trace_command,
    traveler::Traveler,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_travelers(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);
    let list_res = Traveler::db_select(db, msg.chat.id).await;
    match list_res {
        Ok(travelers) => {
            let reply = if travelers.is_empty() {
                translate(ctx, i18n::commands::LIST_TRAVELERS_NOT_FOUND)
            } else {
                travelers
                    .into_iter()
                    .map(|traveler| (*traveler.name).to_owned())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(LOG_DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListTravelers)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, translate_default},
        tests::TestBot,
    };

    test! { list_travelers_ok,
        let db = db().await;

        // Add traveler "Alice"
        let mut bot = TestBot::new(db.clone(), "/addtraveler Alice");
        bot.dispatch().await;
        // Add traveler "Bob"
        bot.update("/addtraveler Bob");
        bot.dispatch().await;

        // List travelers -> "Alice", "Bob"
        bot.update("/listtravelers");
        let response = "Alice\nBob";
        bot.test_last_message(response).await;
    }

    test! { list_travelers_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/listtravelers");
        let response = translate_default(
            i18n::commands::LIST_TRAVELERS_NOT_FOUND,
        );
        bot.test_last_message(&response).await;
    }
}
