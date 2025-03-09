use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::{self, translate_with_args, translate_with_args_default},
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_expense(
    msg: &Message,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);

    // Check if expense exists on db
    let count_res = Expense::db_count(msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete expense from db
            let delete_res = Expense::db_delete(msg.chat.id, number).await;
            match delete_res {
                Ok(_) => {
                    tracing::debug!(DEBUG_SUCCESS);
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::DELETE_EXPENSE_OK,
                        &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::DELETE_EXPENSE_NOT_FOUND,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::DELETE_EXPENSE_NOT_FOUND,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteExpense { number })
        }
    }
}
