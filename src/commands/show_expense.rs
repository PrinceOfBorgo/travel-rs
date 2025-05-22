use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    expense_details::ExpenseDetails,
    i18n::{self, Translate, translate_with_args, translate_with_args_default},
    money_wrapper::MoneyWrapper,
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_expense(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);

    // Check if expense exists on db
    let count_res = Expense::db_count(db.clone(), msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Retrieve expense details from db
            let select_to_res = ExpenseDetails::expense_details(db, msg.chat.id, number).await;
            match select_to_res {
                Ok(Some(ExpenseDetails {
                    expense_number,
                    expense_description,
                    expense_amount,
                    creditor_name,
                    shares,
                    ..
                })) => {
                    let amount = MoneyWrapper::new_with_context(expense_amount, ctx.clone());
                    let reply = translate_with_args(
                        ctx.clone(),
                        i18n::commands::SHOW_EXPENSE_OK,
                        &hashmap! {
                            i18n::args::NUMBER.into() => expense_number.to_string().into(),
                            i18n::args::DESCRIPTION.into() => expense_description.to_string().into(),
                            i18n::args::AMOUNT.into() => amount.to_string().into(),
                            i18n::args::CREDITOR.into() => creditor_name.to_string().into(),
                            i18n::args::SHARES.into() => shares
                                .into_iter()
                                .map(|share_details| share_details.translate(ctx.clone()))
                                .collect::<Vec<_>>()
                                .join("\n").into(),
                        },
                    );
                    tracing::debug!(LOG_DEBUG_SUCCESS);
                    Ok(reply)
                }
                Ok(_) => {
                    tracing::warn!(
                        "{}",
                        translate_with_args_default(
                            i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                            &hashmap! {i18n::args::NUMBER.into() => number.into()},
                        )
                    );
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                        &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::ShowExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::SHOW_EXPENSE_NOT_FOUND,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowExpense { number })
        }
    }
}
