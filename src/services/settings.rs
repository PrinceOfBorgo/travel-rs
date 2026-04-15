use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};
use teloxide::types::ChatId;
use unic_langid::LanguageIdentifier;

use crate::i18n;
use crate::tables::chat::Chat;

use super::ServiceError;

/// Update the currency for a chat.
pub async fn set_currency(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    currency: &str,
) -> Result<String, ServiceError> {
    let currency = currency.trim().to_uppercase();
    if currency.is_empty() {
        return Err(ServiceError::EmptyInput("Currency"));
    }

    Chat::db_update_currency(db, chat_id, &currency).await?;
    Ok(currency)
}

/// Update the language for a chat. Validates that the language is available.
pub async fn set_language(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    langid: &LanguageIdentifier,
) -> Result<(), ServiceError> {
    if !i18n::is_lang_available(langid) {
        let available: Vec<String> = i18n::available_langs().map(|l| l.to_string()).collect();
        return Err(ServiceError::LanguageNotAvailable {
            requested: langid.to_string(),
            available,
        });
    }

    Chat::db_update_lang(db, chat_id, langid).await?;
    Ok(())
}
