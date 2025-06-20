use crate::{
    Context,
    chat::Chat,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, TranslateWithArgs},
    trace_command_db,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;
use unic_langid::LanguageIdentifier;

#[apply(trace_command_db)]
pub async fn set_language(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    langid: LanguageIdentifier,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");
    // Check if language is available
    if !i18n::is_lang_available(&langid) {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(
            i18n::commands::SET_LANGUAGE_NOT_AVAILABLE.translate_with_args(
                ctx,
                &hashmap! {
                    i18n::args::LANGID.into() => langid.to_string().into(),
                    i18n::args::AVAILABLE_LANGS.into() =>
                        i18n::available_langs()
                        .map(|lang| format!("- {lang}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into(),
                },
            ),
        );
    }

    // Update chat language on db
    let update_res = Chat::db_update_lang(db, msg.chat.id, &langid).await;
    match update_res {
        Ok(_) => {
            tracing::debug!("{LOG_DEBUG_SUCCESS}");
            {
                let mut ctx_guard = ctx.lock().expect("Failed to lock context");
                ctx_guard.langid = langid.clone();
            }

            Ok(i18n::commands::SET_LANGUAGE_OK.translate_with_args(
                ctx.clone(),
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
        i18n::{self, TranslateWithArgs},
        tests::TestBot,
    };
    use maplit::hashmap;

    test! { set_language_ok,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setlanguage it-IT");
        let last_message = bot.dispatch_and_last_message().await.unwrap();

        let response = i18n::commands::SET_LANGUAGE_OK.translate_with_args(
            bot.context(),  // Use the new context to retrieve the updated language
            &hashmap! {i18n::args::LANGID.into() => "it-IT".into()},
        );
        // Check that the last message is the expected response
        assert_eq!(last_message, response);
    }

    test! { set_currency_not_available,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setlanguage ab-CD");
        let last_message = bot.dispatch_and_last_message().await.unwrap();

        let response = i18n::commands::SET_LANGUAGE_NOT_AVAILABLE.translate_with_args(
            bot.context(),  // Use the new context to retrieve the updated language
            &hashmap! {
                i18n::args::LANGID.into() => "ab-CD".into(),
                i18n::args::AVAILABLE_LANGS.into() =>
                    i18n::available_langs()
                    .map(|lang| format!("- {lang}"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .into(),
            },
        );
        // Check that the last message is the expected response
        assert_eq!(last_message, response);
    }
}
