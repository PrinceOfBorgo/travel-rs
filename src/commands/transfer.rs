use crate::{
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
    tracing::debug!("START");
    if from.is_empty() || to.is_empty() {
        return Err(CommandError::EmptyInput);
    }
    let chat_id = msg.chat.id;

    // Get sender from db
    let select_from_res = Traveler::db_select_by_name(chat_id, from.clone()).await;
    match select_from_res {
        Ok(senders) if !senders.is_empty() => {
            // Get receiver from db
            let select_to_res = Traveler::db_select_by_name(chat_id, to.clone()).await;
            match select_to_res {
                Ok(recvs) if !recvs.is_empty() => {
                    // Record the new transfer on db
                    let relate_res = TransferredTo::db_relate(
                        amount,
                        senders[0].id.clone(),
                        recvs[0].id.clone(),
                    )
                    .await;
                    match relate_res {
                        Ok(Some(transfer)) => {
                            if let Err(err_update) = update_debts(chat_id).await {
                                tracing::warn!("{err_update}");
                            }
                            tracing::debug!("SUCCESS - id: {}", transfer.id);
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
