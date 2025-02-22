use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    trace_command,
};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_expenses(msg: &Message) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let list_res = Expense::db_select(msg.chat.id).await;
    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                format!(
                    "No expenses found. Use `/{add_expense}` to add one.",
                    add_expense = variant_to_string!(Command::AddExpense)
                )
            } else {
                expenses
                    .into_iter()
                    .map(|expense| format!("{expense}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListExpenses)
        }
    }
}
