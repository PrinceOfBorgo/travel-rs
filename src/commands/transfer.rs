use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, Translate, TranslateWithArgs},
    trace_command_db,
    transferred_to::TransferredTo,
    traveler::{Name, Traveler},
    update_debts,
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
    let chat_id = msg.chat.id;

    // Get sender from db
    let select_from_res = Traveler::db_select_by_name(db.clone(), chat_id, &from).await;
    match select_from_res {
        Ok(Some(sender)) => {
            // Get receiver from db
            let select_to_res = Traveler::db_select_by_name(db.clone(), chat_id, &to).await;
            match select_to_res {
                Ok(Some(recv)) => {
                    // Record the new transfer on db
                    let relate_res =
                        TransferredTo::db_relate(db.clone(), amount, sender.id, recv.id).await;
                    match relate_res {
                        Ok(Some(transfer)) => {
                            if let Err(err_update) = update_debts(db, chat_id).await {
                                tracing::warn!("{err_update}");
                            }
                            tracing::debug!("{LOG_DEBUG_SUCCESS} - id: {}", transfer.id);
                            Ok(i18n::commands::TRANSFER_OK.translate(ctx))
                        }
                        Ok(None) => {
                            let err = CommandError::Transfer {
                                sender: from.to_owned(),
                                receiver: to.to_owned(),
                                amount,
                            };
                            tracing::warn!("{err}");
                            Err(err)
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
                Ok(_) => {
                    tracing::warn!(
                        "{}",
                        i18n::commands::TRANSFER_RECEIVER_NOT_FOUND.translate_with_args_default(
                            &hashmap! {i18n::args::NAME.into() => to.clone().into()},
                        )
                    );
                    Ok(i18n::commands::TRANSFER_RECEIVER_NOT_FOUND
                        .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => to.into()}))
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
        Ok(_) => {
            tracing::warn!(
                "{}",
                i18n::commands::TRANSFER_SENDER_NOT_FOUND.translate_with_args_default(
                    &hashmap! {i18n::args::NAME.into() => from.clone().into()},
                )
            );
            Ok(i18n::commands::TRANSFER_SENDER_NOT_FOUND
                .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => from.into()}))
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
