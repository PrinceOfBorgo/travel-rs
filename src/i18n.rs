use fluent::{FluentResource, FluentValue};
use fluent_templates::Loader;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::RandomState;
use std::sync::LazyLock;
use teloxide::types::ChatId;
use unic_langid::LanguageIdentifier;

use crate::chat::Chat;
use crate::consts::DEFAULT_LANG;

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
        customise: |bundle| {
            static RESOURCES: LazyLock<FluentResource> = LazyLock::new(|| {
                let resources = format!("
-i18n-cancel-command = {cancel}
-i18n-addexpense-command = {add_expense}
-i18n-addtraveler-command = {add_traveler}
", 
cancel = variant_to_string!(Command::Cancel),
add_expense = variant_to_string!(Command::AddExpense),
add_traveler = variant_to_string!(Command::AddTraveler)
);

                FluentResource::try_new(resources).unwrap()
            });

            bundle.add_resource(&RESOURCES).expect("Failed to add FTL resources to the bundle.");
        }
    };
}

async fn get_lang_from_db(chat_id: ChatId) -> LanguageIdentifier {
    let mut lang = DEFAULT_LANG.to_owned();
    match Chat::db_select_by_id(chat_id).await {
        Ok(Some(chat)) => {
            lang = chat.lang;
        }
        Ok(None) => {
            tracing::error!("Error while loading chat with id: {chat_id}")
        }
        Err(err) => tracing::error!("{err}"),
    }
    tracing::debug!("Language for chat {chat_id} is {lang}");
    lang.parse().unwrap_or_else(|_| {
        DEFAULT_LANG
            .parse()
            .unwrap_or_else(|_| panic!("Failed to parse default language {DEFAULT_LANG}"))
    })
}

pub async fn translate(chat_id: ChatId, input: &str) -> String {
    let lang = get_lang_from_db(chat_id).await;
    LOCALES.try_lookup(&lang, input).unwrap_or(input.to_owned())
}

pub async fn translate_with_args(
    chat_id: ChatId,
    input: &str,
    args: &HashMap<Cow<'static, str>, FluentValue<'_>, RandomState>,
) -> String {
    let lang = get_lang_from_db(chat_id).await;
    LOCALES
        .try_lookup_with_args(&lang, input, args)
        .unwrap_or(input.to_owned())
}
