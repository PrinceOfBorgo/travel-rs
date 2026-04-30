//! Generic UI labels (buttons, inline keyboards, etc.) shared across
//! commands and dialogues.

pub const CANCEL_BUTTON: &str = "cancel-button";

/// Prefix for localized language label messages. The full key is built by
/// appending the language identifier (e.g. `language-label-en-US`).
pub const LANGUAGE_LABEL_PREFIX: &str = "language-label-";

/// Prefix for localized currency label messages. The full key is built by
/// appending the currency code (e.g. `currency-label-USD`).
pub const CURRENCY_LABEL_PREFIX: &str = "currency-label-";
