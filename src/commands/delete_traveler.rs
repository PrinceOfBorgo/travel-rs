use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
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
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_traveler(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Retrieve traveler from db
    let select_res = Traveler::db_select_by_name(db.clone(), msg.chat.id, &name).await;
    match select_res {
        Ok(Some(traveler)) => {
            // Check if traveler has paid for expenses
            let list_res = Expense::db_select_by_payer(db.clone(), traveler).await;
            match list_res {
                Ok(expenses) if expenses.is_empty() => {
                    // Delete traveler from db
                    let delete_res = Traveler::db_delete(db.clone(), msg.chat.id, &name).await;
                    match delete_res {
                        Ok(_) => {
                            if let Err(err_update) = update_debts(db, msg.chat.id).await {
                                tracing::warn!("{err_update}");
                            }
                            tracing::debug!(LOG_DEBUG_SUCCESS);
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

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        errors::CommandError,
        expense::Expense,
        i18n::{self, Translate, translate_with_args_default},
        tests::TestBot,
        traveler::{Name, Traveler},
    };
    use maplit::hashmap;
    use std::str::FromStr;
    use teloxide::types::ChatId;

    test! { delete_traveler_ok,
        let db = db().await;

        // Add traveler 'Alice'
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        // Delete traveler 'Alice'
        bot.update("/deletetraveler Alice");
        let response = translate_with_args_default(
            i18n::commands::DELETE_TRAVELER_OK,
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_traveler_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetraveler Alice");
        let response = translate_with_args_default(
            i18n::commands::DELETE_TRAVELER_NOT_FOUND,
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_traveler_twice,
        let db = db().await;

        // Add traveler 'Alice'
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        // Delete traveler 'Alice' -> ok
        bot.update("/deletetraveler Alice");
        let response = translate_with_args_default(
            i18n::commands::DELETE_TRAVELER_OK,
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;

        // Delete traveler 'Alice' again -> not found
        let response = translate_with_args_default(
            i18n::commands::DELETE_TRAVELER_NOT_FOUND,
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_traveler_has_expenses,
        let db = db().await;

        // Add traveler 'Alice'
        let mut bot = TestBot::new(db.clone(), "/addtraveler Alice");
        bot.dispatch().await;

        // Add expense
        bot.update("/addexpense");
        bot.dispatch().await;
        // 1. Set description
        bot.update("Test expense");
        bot.dispatch().await;
        // 2. Set amount
        bot.update("100");
        bot.dispatch().await;
        // 3. Set payer
        bot.update("Alice");
        bot.dispatch().await;
        // 4. Split expense
        bot.update("all");
        bot.dispatch().await;

        // Retrieve traveler 'Alice' and their expenses
        let traveler =
            Traveler::db_select_by_name(db.clone(), ChatId(bot.chat_id()), &Name::from_str("Alice").unwrap())
                .await
                .unwrap()
                .unwrap();
        let expenses = Expense::db_select_by_payer(db, traveler).await.unwrap();
        let expense = expenses.first().unwrap();

        // Delete traveler 'Alice' -> has expenses
        bot.update("/deletetraveler Alice");
        let response = translate_with_args_default(
            i18n::commands::DELETE_TRAVELER_HAS_EXPENSES,
            &hashmap! {
                i18n::args::NAME.into() => "Alice".into(),
                i18n::args::EXPENSES.into() => expense.translate_default().into(),
            },
        );
        bot.test_last_message(&response).await;
    }

    test! { delete_traveler_empty_input,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetraveler");
        let err = CommandError::EmptyInput.translate_default();
        assert!(
            bot.last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );
    }
}
