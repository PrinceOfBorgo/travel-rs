use crate::{
    Context,
    chat::Chat,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, translate_with_args},
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;
use unic_langid::LanguageIdentifier;

#[apply(trace_command)]
pub async fn set_language(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    langid: LanguageIdentifier,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(LOG_DEBUG_START);
    // Check if language is available
    if !i18n::is_lang_available(&langid) {
        tracing::debug!(LOG_DEBUG_SUCCESS);
        return Ok(translate_with_args(
            ctx,
            i18n::commands::SET_LANGUAGE_NOT_AVAILABLE,
            &hashmap! {i18n::args::LANGID.into() => langid.to_string().into()},
        ));
    }

    // Update chat language on db
    let update_res = Chat::db_update_lang(db, msg.chat.id, &langid).await;
    match update_res {
        Ok(_) => {
            tracing::debug!(LOG_DEBUG_SUCCESS);
            {
                let mut ctx_guard = ctx.lock().expect("Failed to lock context");
                ctx_guard.langid = langid.clone();
            }

            Ok(translate_with_args(
                ctx.clone(),
                i18n::commands::SET_LANGUAGE_OK,
                &hashmap! {i18n::args::LANGID.into() => langid.to_string().into()},
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::SetLanguage { langid })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, translate_with_args},
        tests::TestBot,
    };
    use maplit::hashmap;

    test! { set_language_ok,
        let db = db().await;

        // Dispatch once to set the language
        let mut bot = TestBot::new(db, "/setlanguage it-IT");
        bot.dispatch().await;

        // Dispatch again to test if the language has been set correctly
        let response = translate_with_args(
            bot.context(),
            i18n::commands::SET_LANGUAGE_OK,
            &hashmap! {i18n::args::LANGID.into() => "it-IT".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { set_currency_not_available,
        let db = db().await;

        // Dispatch once to set the language
        let mut bot = TestBot::new(db, "/setlanguage ab-CD");

        // Dispatch again to test if the language has been set correctly
        let response = translate_with_args(
            bot.context(),
            i18n::commands::SET_LANGUAGE_NOT_AVAILABLE,
            &hashmap! {i18n::args::LANGID.into() => "ab-CD".into()},
        );
        bot.test_last_message(&response).await;
    }
}
