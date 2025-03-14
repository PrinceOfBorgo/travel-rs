use crate::{
    Context,
    balance::Balance,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, translate, translate_with_args},
    money_wrapper::MoneyWrapper,
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_balances(
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let list_res = Balance::balances(msg.chat.id).await;
    match list_res {
        Ok(balances) => {
            let reply = if balances.is_empty() {
                translate(ctx, i18n::commands::SHOW_BALANCES_SETTLED_UP)
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
                                    i18n::commands::SHOW_BALANCES_OK,
                                    &hashmap! {
                                        i18n::args::DEBTOR.into() => debtor_name.into(),
                                        i18n::args::DEBT.into() => debt.to_string().into(),
                                        i18n::args::CREDITOR.into() => creditor_name.into(),
                                    },
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
            Err(CommandError::ShowBalances)
        }
    }
}
