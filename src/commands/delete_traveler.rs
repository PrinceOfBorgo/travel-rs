use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    expense::Expense,
    i18n::{self, Translate, translate_with_args, translate_with_args_default},
    trace_command,
    traveler::{Name, Traveler},
    utils::update_debts,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_traveler(
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Retrieve traveler from db
    let select_res = Traveler::db_select_by_name(msg.chat.id, &name).await;
    match select_res {
        Ok(Some(traveler)) => {
            // Check if traveler has paid for expenses
            let list_res = Expense::db_select_by_payer(traveler).await;
            match list_res {
                Ok(expenses) if expenses.is_empty() => {
                    // Delete traveler from db
                    let delete_res = Traveler::db_delete(msg.chat.id, &name).await;
                    match delete_res {
                        Ok(_) => {
                            if let Err(err_update) = update_debts(msg.chat.id).await {
                                tracing::warn!("{err_update}");
                            }
                            tracing::debug!(DEBUG_SUCCESS);
                            Ok(translate_with_args(
                                ctx,
                                i18n::commands::DELETE_TRAVELER_OK,
                                &hashmap! {i18n::args::NAME.into() => name.into()},
                            ))
                        }
                        Err(err) => {
                            tracing::error!("{err}");
                            Err(CommandError::DeleteTraveler { name })
                        }
                    }
                }
                Ok(expenses) => {
                    let expenses_reply = expenses
                        .into_iter()
                        .map(|expense| expense.translate(ctx.clone()))
                        .collect::<Vec<_>>()
                        .join("\n");

                    tracing::warn!(
                        "Unable to delete traveler '{name}' because they have associated expenses.",
                    );
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::DELETE_TRAVELER_HAS_EXPENSES,
                        &hashmap! {
                            i18n::args::NAME.into() => name.clone().into(),
                            i18n::args::EXPENSES.into() => expenses_reply.into(),
                        },
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    return Err(CommandError::DeleteTraveler { name });
                }
            }
        }
        Ok(_) => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::DELETE_TRAVELER_NOT_FOUND,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::DELETE_TRAVELER_NOT_FOUND,
                &hashmap! {i18n::args::NAME.into() => name.into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteTraveler {
                name: name.to_owned(),
            })
        }
    }
}
