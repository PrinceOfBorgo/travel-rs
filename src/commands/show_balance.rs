use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    trace_command,
    traveler::Name,
    views::balance::Balance,
};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_balance(msg: &Message, name: Name) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    let list_res = Balance::db_select_by_name(msg.chat.id, name.to_owned()).await;
    match list_res {
        Ok(balances) => {
            let reply = if balances.is_empty() {
                format!("Traveler {name} is settled up with everyone.")
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
                            if debtor_name == name {
                                format!("{name} owes {debt} to {creditor_name}.")
                            } else {
                                format!("{name} is owed {debt} from {debtor_name}.")
                            }
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
            Err(CommandError::ShowBalance { name })
        }
    }
}
