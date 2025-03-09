use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translatable, translate},
    trace_command,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_expenses(
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let list_res = Expense::db_select(msg.chat.id).await;
    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                translate(ctx, i18n::commands::LIST_EXPENSES_NOT_FOUND)
            } else {
                expenses
                    .into_iter()
                    .map(|expense| expense.translate(ctx.clone()))
                    .collect::<Vec<_>>()
                    .join("\n\n")
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
