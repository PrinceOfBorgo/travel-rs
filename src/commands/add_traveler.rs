use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, TranslateWithArgs},
    trace_command_db,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn add_traveler(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(db.clone(), msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            tracing::warn!(
                "{}",
                i18n::commands::ADD_TRAVELER_ALREADY_ADDED.translate_with_args_default(
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                )
            );
            Ok(i18n::commands::ADD_TRAVELER_ALREADY_ADDED
                .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => name.into()}))
        }
        Ok(_) => {
            // Create traveler on db
            let create_res = Traveler::db_create(db, msg.chat.id, &name).await;
            match create_res {
                Ok(_) => {
                    tracing::debug!("{LOG_DEBUG_SUCCESS}");
                    Ok(i18n::commands::ADD_TRAVELER_OK.translate_with_args(
                        ctx,
                        &hashmap! {i18n::args::NAME.into() => name.into()},
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::AddTraveler { name })
                }
            }
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::AddTraveler {
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
        i18n::{self, Translate, TranslateWithArgs},
        tests::TestBot,
    };
    use maplit::hashmap;

    test! { add_traveler_ok,
        let db = db().await;

        let mut bot = TestBot::new(db, "/addtraveler Alice");
        let response = i18n::commands::ADD_TRAVELER_OK.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { add_traveler_already_added,
        let db = db().await;

        // Add traveler "Alice"
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        // Try to add traveler "Alice" again
        let response = i18n::commands::ADD_TRAVELER_ALREADY_ADDED.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { add_traveler_empty_input,
        let db = db().await;

        let mut bot = TestBot::new(db, "/addtraveler");
        let err = CommandError::EmptyInput.translate_default();
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );
    }
}
