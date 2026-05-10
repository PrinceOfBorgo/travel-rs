//! Yes / No confirmation inline keyboard and text-input helpers.

use crate::{
    Context,
    consts::{CONFIRM_NO_KWORDS, CONFIRM_YES_KWORDS},
    i18n::{self, Translate},
};
use std::sync::{Arc, Mutex};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Configuration for [`confirmation_keyboard`].
pub struct ConfirmConfig<'a> {
    /// Callback data sent when the user presses "Yes".
    pub confirm_callback: &'a str,
    /// Callback data sent when the user presses "No".
    pub deny_callback: &'a str,
    /// Translation context.
    pub ctx: Arc<Mutex<Context>>,
}

/// Builds a one-row inline keyboard with localized **Yes** and **No** buttons.
pub fn confirmation_keyboard(config: ConfirmConfig<'_>) -> InlineKeyboardMarkup {
    let yes_label = i18n::labels::CONFIRM_YES_BUTTON.translate(config.ctx.clone());
    let no_label = i18n::labels::CONFIRM_NO_BUTTON.translate(config.ctx);

    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback(yes_label, config.confirm_callback),
        InlineKeyboardButton::callback(no_label, config.deny_callback),
    ]])
}

/// Result of matching user text against confirmation keywords.
pub enum ConfirmAnswer {
    Yes,
    No,
    Unknown,
}

/// Matches trimmed, case-insensitive text against the accepted yes/no keywords.
pub fn parse_confirm_answer(text: &str) -> ConfirmAnswer {
    let lower = text.trim().to_lowercase();
    if CONFIRM_YES_KWORDS.iter().any(|&k| k == lower) {
        ConfirmAnswer::Yes
    } else if CONFIRM_NO_KWORDS.iter().any(|&k| k == lower) {
        ConfirmAnswer::No
    } else {
        ConfirmAnswer::Unknown
    }
}
