use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, translate, translate_with_args, translate_with_args_default},
    trace_command,
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

#[apply(trace_command)]
pub async fn transfer(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    from: Name,
    to: Name,
    amount: Decimal,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
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
                            tracing::debug!("{DEBUG_SUCCESS} - id: {}", transfer.id);
                            Ok(translate(ctx, i18n::commands::TRANSFER_OK))
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
                        translate_with_args_default(
                            i18n::commands::TRANSFER_RECEIVER_NOT_FOUND,
                            &hashmap! {i18n::args::NAME.into() => to.clone().into()},
                        )
                    );
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::TRANSFER_RECEIVER_NOT_FOUND,
                        &hashmap! {i18n::args::NAME.into() => to.into()},
                    ))
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
                translate_with_args_default(
                    i18n::commands::TRANSFER_SENDER_NOT_FOUND,
                    &hashmap! {i18n::args::NAME.into() => from.clone().into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::TRANSFER_SENDER_NOT_FOUND,
                &hashmap! {i18n::args::NAME.into() => from.into()},
            ))
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
