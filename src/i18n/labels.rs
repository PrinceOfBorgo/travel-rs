//! Generic UI labels (buttons, inline keyboards, etc.) shared across
//! commands and dialogues.

pub const CANCEL_BUTTON: &str = "cancel-button";
pub const ALL_BUTTON: &str = "all-button";
pub const END_BUTTON: &str = "end-button";
pub const FILTER_BUTTON: &str = "filter-button";
pub const HELP_BUTTON: &str = "help-button";

/// Prefix for localized language label messages. The full key is built by
/// appending the language identifier (e.g. `language-label-en-US`).
pub const LANGUAGE_LABEL_PREFIX: &str = "language-label-";
