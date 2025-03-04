use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{translate, translate_with_args},
    trace_command,
    transferred_to::TransferredTo,
    traveler::{Name, Traveler},
    update_debts,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn transfer(
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
    let select_from_res = Traveler::db_select_by_name(chat_id, from.clone()).await;
    match select_from_res {
        Ok(Some(sender)) => {
            // Get receiver from db
            let select_to_res = Traveler::db_select_by_name(chat_id, to.clone()).await;
            match select_to_res {
                Ok(Some(recv)) => {
                    // Record the new transfer on db
                    let relate_res = TransferredTo::db_relate(amount, sender.id, recv.id).await;
                    match relate_res {
                        Ok(Some(transfer)) => {
                            if let Err(err_update) = update_debts(chat_id).await {
                                tracing::warn!("{err_update}");
                            }
                            tracing::debug!("{DEBUG_SUCCESS} - id: {}", transfer.id);
                            Ok(translate(ctx, "i18n-transfer-ok"))
                        }
                        Ok(None) => {
                            tracing::warn!("Couldn't record the transfer.");
                            Err(CommandError::Transfer {
                                from: from.to_owned(),
                                to: to.to_owned(),
                                amount,
                            })
                        }
                        Err(err) => {
                            tracing::error!("{err}");
                            Err(CommandError::Transfer {
                                from: from.to_owned(),
                                to: to.to_owned(),
                                amount,
                            })
                        }
                    }
                }
                Ok(_) => {
                    tracing::warn!("Couldn't find traveler \"{to}\" to transfer money to.");
                    Ok(translate_with_args(
                        ctx,
                        "i18n-transfer-receiver-not-found",
                        &hashmap! {"name".into() => to.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::Transfer {
                        from: from.to_owned(),
                        to: to.to_owned(),
                        amount,
                    })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find traveler \"{from}\" to transfer money from.");
            Ok(translate_with_args(
                ctx,
                "i18n-transfer-sender-not-found",
                &hashmap! {"name".into() => from.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::Transfer {
                from: from.to_owned(),
                to: to.to_owned(),
                amount,
            })
        }
    }
}
