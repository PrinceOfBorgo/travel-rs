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
pub async fn find_expenses(msg: &Message, description: &str) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if description.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    let list_res = Expense::db_select_by_descr(msg.chat.id, description.to_owned()).await;
    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                translate_with_args(
                    msg.chat.id,
                    "i18n-find-expenses-not-found",
                    &hashmap! {"description".into() => description.into()},
                )
                .await
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
            Err(CommandError::FindExpenses {
                description: description.to_owned(),
            })
        }
    }
}
