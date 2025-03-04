use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::translate_with_args,
    trace_command,
    traveler::{Name, Traveler},
    views::balance::Balance,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use teloxide::prelude::*;
use tracing::Level;
use std::sync::{Arc, Mutex};

#[apply(trace_command)]
pub async fn show_balance(
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Retrieve balances from db
            let list_res = Balance::db_select_by_name(msg.chat.id, name.to_owned()).await;
            match list_res {
                Ok(balances) => {
                    let reply = if balances.is_empty() {
                        translate_with_args(
                            ctx,
                            "i18n-show-balance-settled-up",
                            &hashmap! {"name".into() => name.into()},
                        )
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
                                    translate_with_args(
                                        ctx.clone(),
                                        "i18n-show-balance-ok",
                                        &hashmap!{
                                            "traveler-name".into() => name.clone().into(),
                                            "traveler-is".into() => if debtor_name == name { "debtor" } else { "creditor" }.into(),
                                            "debt".into() => debt.to_string().into(),
                                            "other-traveler-name".into() => if debtor_name == name { creditor_name } else { debtor_name }.into(),
                                        },
                                    )
                                    
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
        Ok(_) => {
            tracing::warn!("Couldn't find traveler {name} to show the balance.");
            Ok(translate_with_args(
                ctx,
                "i18n-show-balance-traveler-not-found",
                &hashmap! {"name".into() => name.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowBalance { name })
        }
    }
}
