use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, Translate, translate, translate_with_args},
    trace_command,
    transfer::Transfer,
    traveler::Name,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_transfers(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);

    let list_res = if name.is_empty() {
        Transfer::transfers(db, msg.chat.id).await
    } else {
        Transfer::transfers_by_name(db, msg.chat.id, name.clone()).await
    };

    match list_res {
        Ok(transfers) => {
            let reply = if transfers.is_empty() {
                if name.is_empty() {
                    translate(ctx, i18n::commands::LIST_TRANSFERS_NOT_FOUND)
                } else {
                    translate_with_args(
                        ctx,
                        i18n::commands::LIST_TRANSFERS_NAME_NOT_FOUND,
                        &hashmap! {i18n::args::NAME.into() => name.into()},
                    )
                }
            } else {
                transfers
                    .into_iter()
                    .map(|transfer| transfer.translate(ctx.clone()))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(LOG_DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListTransfers { name })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate, translate_default, translate_with_args_default},
        tests::{TestBot, helpers},
        transfer::Transfer,
        traveler::Name,
    };
    use maplit::hashmap;
    use std::str::FromStr;

    test! { list_transfers_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob" and "Charlie"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;

        // Transfer 100 from Alice to Bob
        helpers::transfer(&mut bot, "Alice", "Bob", 100.into()).await;
        // Transfer 50 from Bob to Charlie
        helpers::transfer(&mut bot, "Bob", "Charlie", 50.into()).await;

        // List transfers
        bot.update("/listtransfers");
        let transfers = Transfer::transfers(db, bot.chat_id()).await.unwrap();
        // Check that two transfers has been recorded
        assert_eq!(transfers.len(), 2);
        let response = transfers
            .into_iter()
            .map(|transfer| transfer.translate_default())
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;
    }

    test! { list_transfers_by_name_ok,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob" and "Charlie"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;

        // Transfer 100 from Alice to Bob
        helpers::transfer(&mut bot, "Alice", "Bob", 100.into()).await;
        // Transfer 50 from Bob to Charlie
        helpers::transfer(&mut bot, "Bob", "Charlie", 50.into()).await;

        // List transfers related to Alice
        bot.update("/listtransfers Alice");
        let transfers = Transfer::transfers_by_name(db.clone(), bot.chat_id(), Name::from_str("Alice").unwrap()).await.unwrap();
        // Check that only one transfer is returned
        assert_eq!(transfers.len(), 1);
        let response = transfers
            .into_iter()
            .map(|transfer| transfer.translate_default())
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;

        // List transfers related to Bob
        bot.update("/listtransfers Bob");
        let transfers = Transfer::transfers_by_name(db, bot.chat_id(), Name::from_str("Bob").unwrap()).await.unwrap();
        // Check that two transfers are returned
        assert_eq!(transfers.len(), 2);
        let response = transfers
            .into_iter()
            .map(|transfer| transfer.translate_default())
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;
    }

    test! { list_transfers_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/listtransfers");
        let response = translate_default(
            i18n::commands::LIST_TRANSFERS_NOT_FOUND,
        );
        bot.test_last_message(&response).await;
    }

    test! { list_transfers_name_not_found,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice" and "Bob"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Transfer 100 from Alice to Bob
        helpers::transfer(&mut bot, "Alice", "Bob", 100.into()).await;

        // List transfers related to Charlie -> not found
        bot.update("/listtransfers Charlie");
        let response = translate_with_args_default(
            i18n::commands::LIST_TRANSFERS_NAME_NOT_FOUND,
            &hashmap! {i18n::args::NAME.into() => "Charlie".into()},
        );
        bot.test_last_message(&response).await;
    }
}
