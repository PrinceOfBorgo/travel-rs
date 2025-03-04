use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::translate_with_args,
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_expense(msg: &Message, number: i64) -> Result<String, CommandError> {
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
                        msg.chat.id,
                        "i18n-delete-expense-ok",
                        &hashmap!["number".into() => number.into()],
                    )
                    .await)
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find expense #{number} to delete.");
            Ok(translate_with_args(
                msg.chat.id,
                "i18n-delete-expense-not-found",
                &hashmap!["number".into() => number.into()],
            )
            .await)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteExpense { number })
        }
    }
}
