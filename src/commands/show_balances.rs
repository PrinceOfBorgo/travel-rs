use crate::{
    Context,
    balance::Balance,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{
        self,
        args::{TRAVELER_IS_CASE_CREDITOR, TRAVELER_IS_CASE_DEBTOR},
        translate, translate_with_args, translate_with_args_default,
    },
    money_wrapper::MoneyWrapper,
    trace_command,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_balances(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);
    let res = if name.is_empty() {
        show_balances_no_name(db, msg, ctx).await
    } else {
        // Check if traveler exists on db
        let count_res = Traveler::db_count(db.clone(), msg.chat.id, &name).await;
        match count_res {
            Ok(Some(count)) if *count > 0 => {
                show_balances_with_name(db, msg, name.clone(), ctx).await
            }
            Ok(_) => {
                tracing::warn!(
                    "{}",
                    translate_with_args_default(
                        i18n::commands::SHOW_BALANCES_TRAVELER_NOT_FOUND,
                        &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    )
                );
                return Ok(translate_with_args(
                    ctx,
                    i18n::commands::SHOW_BALANCES_TRAVELER_NOT_FOUND,
                    &hashmap! {i18n::args::NAME.into() => name.into()},
                ));
            }
            Err(err) => {
                tracing::error!("{err}");
                return Err(CommandError::ShowBalances { name });
            }
        }
    };

    match res {
        Ok(reply) => {
            tracing::debug!(LOG_DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowBalances { name })
        }
    }
}

async fn show_balances_no_name(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, surrealdb::Error> {
    let list_res = Balance::balances(db, msg.chat.id).await;
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
            Ok(reply)
        }
        Err(err) => Err(err),
    }
}

pub async fn show_balances_with_name(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, surrealdb::Error> {
    // Retrieve balances from db
    let list_res = Balance::balances_by_name(db, msg.chat.id, name.to_owned()).await;
    match list_res {
        Ok(balances) => {
            let reply = if balances.is_empty() {
                translate_with_args(
                    ctx,
                    i18n::commands::SHOW_BALANCES_TRAVELER_SETTLED_UP,
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
                                            i18n::commands::SHOW_BALANCES_TRAVELER_OK,
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
            Ok(reply)
        }
        Err(err) => Err(err),
    }
}
