use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    expense_details::ExpenseDetails,
    i18n::{self, Translatable, translate_with_args, translate_with_args_default},
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_expense(
    msg: &Message,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);

    // Check if expense exists on db
    let count_res = Expense::db_count(msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Retrieve expense details from db
            let select_to_res = ExpenseDetails::expense_details(msg.chat.id, number).await;
            match select_to_res {
                Ok(Some(ExpenseDetails {
                    expense_number,
                    expense_description,
                    expense_amount,
                    creditor_name,
                    shares,
                    ..
                })) => {
                    let reply = translate_with_args(
                        ctx.clone(),
                        i18n::commands::SHOW_EXPENSE_OK,
                        &hashmap! {
                            i18n::args::NUMBER.into() => expense_number.to_string().into(),
                            i18n::args::DESCRIPTION.into() => expense_description.to_string().into(),
                            i18n::args::AMOUNT.into() => expense_amount.to_string().into(),
                            i18n::args::CREDITOR.into() => creditor_name.to_string().into(),
                            i18n::args::SHARES.into() => shares
                                .into_iter()
                                .map(|share_details| share_details.translate(ctx.clone()))
                                .collect::<Vec<_>>()
                                .join("\n").into(),
                        },
                    );
                    tracing::debug!(DEBUG_SUCCESS);
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
