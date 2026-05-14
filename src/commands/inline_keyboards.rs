//! Generic inline-keyboard builders and a stateless callback dispatcher.
//!
//! This module provides reusable building blocks for commands that attach an
//! inline keyboard to their response. The keyboard contents and the
//! callback-to-command mapping are supplied by the caller — this module does
//! not reference specific `Command` variants.

use crate::{
    Context,
    commands::{Command, command_reply},
    traveler::Traveler,
};
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup},
};

// ─── Generic keyboard builder ────────────────────────────────────────────────

/// Builds an [`InlineKeyboardMarkup`] laying out `buttons` in rows of
/// `per_row` columns.
pub fn buttons_keyboard(
    buttons: Vec<InlineKeyboardButton>,
    per_row: usize,
) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = buttons
        .chunks(per_row)
        .map(<[InlineKeyboardButton]>::to_vec)
        .collect();
    InlineKeyboardMarkup::new(rows)
}

// ─── Stateless callback dispatcher ──────────────────────────────────────────

/// A registered stateless callback: a prefix to match and a function that
/// maps the stripped value (after the prefix) to a `Command`.
pub struct CallbackMapping {
    /// The prefix that identifies this callback (e.g. `"help:"`).
    pub prefix: &'static str,
    /// Builds a `Command` from the value stripped of the prefix.
    /// Returns `None` if the value is invalid (callback is silently ignored).
    pub to_command: fn(&str) -> Option<Command>,
    /// When `true`, the raw callback value is a traveler number that
    /// must be resolved to a name via DB before calling [`Self::to_command`].
    pub resolve_traveler_number: bool,
}

/// Returns `true` if `data` matches any of the registered stateless prefixes.
pub fn is_stateless_callback(data: &str, mappings: &[CallbackMapping]) -> bool {
    mappings.iter().any(|m| data.starts_with(m.prefix))
}

/// Generic stateless callback handler. Answers the query, removes the
/// keyboard, resolves the callback data via `mappings`, and sends the
/// command reply.
pub async fn handle_stateless_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
    mappings: &[CallbackMapping],
) -> crate::HandlerResult {
    let _ = bot.answer_callback_query(q.id.clone()).await;

    let Some(msg) = q.regular_message().cloned() else {
        return Ok(());
    };

    let data = q.data.as_deref().unwrap_or("");

    // Ignore blank spacer buttons (noop).
    if data.ends_with("__noop__") {
        return Ok(());
    }

    // Edit the original message to show which option was selected and
    // remove the keyboard.
    let selected_value = mappings.iter().find_map(|m| data.strip_prefix(m.prefix));
    if let (Some(val), Some(text)) = (selected_value, msg.text()) {
        // For traveler-number callbacks, resolve the name for the echo;
        // for other callbacks, echo the raw value.
        let display_val =
            if let Some(mapping) = mappings.iter().find(|m| data.starts_with(m.prefix)) {
                if mapping.resolve_traveler_number {
                    Traveler::db_resolve_by_number(db.clone(), msg.chat.id, val)
                        .await
                        .map(|t| t.name.to_string())
                        .unwrap_or_else(|| val.to_owned())
                } else {
                    val.to_owned()
                }
            } else {
                val.to_owned()
            };
        let _ = bot
            .edit_message_text(msg.chat.id, msg.id, format!("{text}\n✓ {display_val}"))
            .await;
    }
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    for mapping in mappings {
        if let Some(value) = data.strip_prefix(mapping.prefix) {
            // Resolve traveler number to name if needed.
            let resolved_value;
            let effective_value = if mapping.resolve_traveler_number {
                resolved_value =
                    Traveler::db_resolve_by_number(db.clone(), msg.chat.id, value)
                        .await
                        .map(|t| t.name.to_string());
                match resolved_value.as_deref() {
                    Some(name) => name,
                    None => {
                        tracing::warn!(
                            "Could not resolve traveler number from callback data: {value:?}"
                        );
                        break;
                    }
                }
            } else {
                value
            };
            if let Some(cmd) = (mapping.to_command)(effective_value) {
                let outcome = command_reply(db, &msg, &cmd, ctx).await;
                bot.send_message(msg.chat.id, outcome.into_message())
                    .await?;
            }
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use teloxide::types::InlineKeyboardButton;

    #[test]
    fn buttons_keyboard_single_row() {
        let buttons = vec![
            InlineKeyboardButton::callback("A", "a"),
            InlineKeyboardButton::callback("B", "b"),
        ];
        let kb = buttons_keyboard(buttons, 3);
        assert_eq!(kb.inline_keyboard.len(), 1);
        assert_eq!(kb.inline_keyboard[0].len(), 2);
    }

    #[test]
    fn buttons_keyboard_multiple_rows() {
        let buttons = vec![
            InlineKeyboardButton::callback("A", "a"),
            InlineKeyboardButton::callback("B", "b"),
            InlineKeyboardButton::callback("C", "c"),
            InlineKeyboardButton::callback("D", "d"),
            InlineKeyboardButton::callback("E", "e"),
        ];
        let kb = buttons_keyboard(buttons, 2);
        assert_eq!(kb.inline_keyboard.len(), 3);
        assert_eq!(kb.inline_keyboard[0].len(), 2);
        assert_eq!(kb.inline_keyboard[1].len(), 2);
        assert_eq!(kb.inline_keyboard[2].len(), 1);
    }

    #[test]
    fn buttons_keyboard_empty() {
        let kb = buttons_keyboard(vec![], 2);
        assert!(kb.inline_keyboard.is_empty());
    }

    #[test]
    fn is_stateless_callback_matches_prefix() {
        let mappings = vec![
            CallbackMapping {
                prefix: "foo:",
                to_command: |_| None,
                resolve_traveler_number: false,
            },
            CallbackMapping {
                prefix: "bar:",
                to_command: |_| None,
                resolve_traveler_number: false,
            },
        ];
        assert!(is_stateless_callback("foo:something", &mappings));
        assert!(is_stateless_callback("bar:", &mappings));
        assert!(!is_stateless_callback("baz:nope", &mappings));
        assert!(!is_stateless_callback("", &mappings));
    }

    #[test]
    fn is_stateless_callback_empty_mappings() {
        assert!(!is_stateless_callback("anything", &[]));
    }
}
