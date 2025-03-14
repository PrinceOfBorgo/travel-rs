use crate::{
    Context,
    balance::Balance,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{
        self,
        args::{TRAVELER_IS_CASE_CREDITOR, TRAVELER_IS_CASE_DEBTOR},
        translate_with_args, translate_with_args_default,
    },
    money_wrapper::MoneyWrapper,
    trace_command,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

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
            let list_res = Balance::balances_by_name(msg.chat.id, name.to_owned()).await;
            match list_res {
                Ok(balances) => {
                    let reply = if balances.is_empty() {
                        translate_with_args(
                            ctx,
                            i18n::commands::SHOW_BALANCE_SETTLED_UP,
                            &hashmap! {i18n::args::NAME.into() => name.into()},
                        )
                    } else {
                        let currency = ctx.lock().expect("Failed to lock context").currency.clone();
                        balances
                            .into_iter()
                            .filter_map(
                                |Balance {
                                     debtor_name,
                                     creditor_name,
                                     debt,
                                     ..
                                 }| {
                                    let debt = MoneyWrapper::new(debt, &currency);
                                    if debt.round_value().is_zero() {
                                        None
                                    } else {
                                        Some(translate_with_args(
                                            ctx.clone(),
                                            i18n::commands::SHOW_BALANCE_OK,
                                            &hashmap! {
                                                i18n::args::TRAVELER_NAME.into() => name.clone().into(),
                                                i18n::args::TRAVELER_IS.into() => if debtor_name == name { TRAVELER_IS_CASE_DEBTOR } else { TRAVELER_IS_CASE_CREDITOR }.into(),
                                                i18n::args::DEBT.into() => debt.to_string().into(),
                                                i18n::args::OTHER_TRAVELER_NAME.into() => if debtor_name == name { creditor_name } else { debtor_name }.into(),
                                            }
                                        ))
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
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::SHOW_BALANCE_TRAVELER_NOT_FOUND,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::SHOW_BALANCE_TRAVELER_NOT_FOUND,
                &hashmap! {i18n::args::NAME.into() => name.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowBalance { name })
        }
    }
}
