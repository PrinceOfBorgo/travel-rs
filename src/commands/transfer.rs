use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    trace_command,
    transferred_to::TransferredTo,
    traveler::{Name, Traveler},
    update_debts,
};
use macro_rules_attribute::apply;
use rust_decimal::Decimal;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn transfer(
    msg: &Message,
    from: Name,
    to: Name,
    amount: Decimal,
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
                            Ok(String::from("Transfer recorded successfully."))
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
                    Ok(format!(
                        "Couldn't find traveler \"{to}\" to transfer money to."
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
            Ok(format!(
                "Couldn't find traveler \"{from}\" to transfer money from."
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
