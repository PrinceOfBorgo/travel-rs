use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, Translate, TranslateWithArgs},
    services,
    trace_command_db,
    traveler::Name,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn transfer(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    from: Name,
    to: Name,
    amount: Decimal,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");
    if from.is_empty() || to.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    match services::transfer::create_transfer(db, msg.chat.id, &from, &to, amount).await {
        Ok(transfer) => {
            tracing::debug!("{LOG_DEBUG_SUCCESS} - id: {}", transfer.id);
            Ok(i18n::commands::TRANSFER_OK.translate(ctx))
        }
        Err(services::ServiceError::NotFound(what)) => {
            // Distinguish sender vs receiver not found
            if what == "Sender" {
                tracing::warn!(
                    "{}",
                    i18n::commands::TRANSFER_SENDER_NOT_FOUND.translate_with_args_default(
                        &hashmap! {i18n::args::NAME.into() => from.clone().into()},
                    )
                );
                Ok(i18n::commands::TRANSFER_SENDER_NOT_FOUND
                    .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => from.into()}))
            } else {
                tracing::warn!(
                    "{}",
                    i18n::commands::TRANSFER_RECEIVER_NOT_FOUND.translate_with_args_default(
                        &hashmap! {i18n::args::NAME.into() => to.clone().into()},
                    )
                );
                Ok(i18n::commands::TRANSFER_RECEIVER_NOT_FOUND
                    .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => to.into()}))
            }
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::Transfer {
                sender: from.to_owned(),
                receiver: to.to_owned(),
                amount,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        errors::CommandError,
        i18n::{self, Translate, TranslateWithArgs},
        tests::{TestBot, helpers},
    };
    use maplit::hashmap;

    test! { transfer_ok,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        // Add travelers "Alice" and "Bob"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Transfer 100 from Alice to Bob
        bot.update("/transfer Alice Bob 100");
        let response = i18n::commands::TRANSFER_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { transfer_receiver_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        // Add traveler "Alice"
        helpers::add_traveler(&mut bot, "Alice").await;

        // Try to transfer 100 from Alice to Bob -> Bob not found
        bot.update("/transfer Alice Bob 100");
        let response = i18n::commands::TRANSFER_RECEIVER_NOT_FOUND.translate_with_args_default(&hashmap! {i18n::args::NAME.into() => "Bob".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { transfer_sender_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        // Add traveler "Bob"
        helpers::add_traveler(&mut bot, "Bob").await;

        // Try to transfer 100 from Alice to Bob -> Alice not found
        bot.update("/transfer Alice Bob 100");
        let response = i18n::commands::TRANSFER_SENDER_NOT_FOUND.translate_with_args_default(&hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { transfer_empty_input,
        let db = db().await;

        // Missing receiver
        let mut bot = TestBot::new(db, "/transfer Alice  100");
        let err = CommandError::EmptyInput.translate_default();
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );

        // Missing sender
        bot.update("/transfer  Bob 100");
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );

        // Missing both
        bot.update("/transfer   100");
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );
    }
}
