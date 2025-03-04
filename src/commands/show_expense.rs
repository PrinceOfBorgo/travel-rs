use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    expense_details::{ExpenseDetails, ShareDetails},
    i18n::translate_with_args,
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn show_expense(msg: &Message, number: i64) -> Result<String, CommandError> {
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
                        msg.chat.id,
                        "i18n-show-expense-ok",
                        &hashmap! {
                            "number".into() => expense_number.to_string().into(),
                            "description".into() => expense_description.to_string().into(),
                            "amount".into() => expense_amount.to_string().into(),
                            "creditor".into() => creditor_name.to_string().into(),
                            "shares".into() => shares
                                .into_iter()
                                .map(
                                    |ShareDetails {
                                        traveler_name,
                                        amount,
                                    }| {
                                        format!("- {traveler_name}: {amount}")
                                    },
                                )
                                .collect::<Vec<_>>()
                                .join("\n").into(),
                        },
                    )
                    .await;
                    tracing::debug!(DEBUG_SUCCESS);
                    Ok(reply)
                }
                Ok(_) => {
                    tracing::warn!("Couldn't find expense #{number} to show the details.");
                    Ok(translate_with_args(
                        msg.chat.id,
                        "i18n-show-expense-not-found",
                        &hashmap! {"number".into() => number.into()},
                    )
                    .await)
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::ShowExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find expense #{number} to show the details.");
            Ok(translate_with_args(
                msg.chat.id,
                "i18n-show-expense-not-found",
                &hashmap! {"number".into() => number.into()},
            )
            .await)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowExpense { number })
        }
    }
}
