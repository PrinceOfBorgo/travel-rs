use crate::{errors::CommandError, expense::Expense, trace_command};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_expense(msg: &Message, number: i64) -> Result<String, CommandError> {
    tracing::debug!("START");

    // Check if expense exists on db
    let count_res = Expense::db_count(msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete expense from db
            let delete_res = Expense::db_delete(msg.chat.id, number).await;
            match delete_res {
                Ok(_) => {
                    tracing::debug!("SUCCESS");
                    Ok(format!("Expense #{number} deleted successfully."))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find expense #{number} to delete.");
            Ok(format!("Couldn't find expense #{number} to delete."))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteExpense { number })
        }
    }
}
