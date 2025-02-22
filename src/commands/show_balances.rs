use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    trace_command,
    views::balance::Balance,
};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_balances(msg: &Message) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let list_res = Balance::db_select(msg.chat.id).await;
    match list_res {
        Ok(balances) => {
            let reply = if balances.is_empty() {
                String::from("All travelers are settled up with everyone.")
            } else {
                balances
                    .into_iter()
                    .map(
                        |Balance {
                             debtor_name,
                             creditor_name,
                             debt,
                             ..
                         }| {
                            format!("{debtor_name} owes {debt} to {creditor_name}.")
                        },
                    )
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowBalances)
        }
    }
}
