use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translate, translate, translate_with_args},
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_expenses(
    msg: &Message,
    description: &str,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let list_res = if description.is_empty() {
        Expense::db_select(msg.chat.id).await
    } else {
        Expense::db_select_by_descr(msg.chat.id, description.to_owned()).await
    };
    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                if description.is_empty() {
                    translate(ctx, i18n::commands::LIST_EXPENSES_NOT_FOUND)
                } else {
                    translate_with_args(
                        ctx,
                        i18n::commands::LIST_EXPENSES_DESCR_NOT_FOUND,
                        &hashmap! {i18n::args::DESCRIPTION.into() => description.into()},
                    )
                }
            } else {
                expenses
                    .into_iter()
                    .map(|expense| expense.translate(ctx.clone()))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListExpenses {
                description: description.to_owned(),
            })
        }
    }
}
