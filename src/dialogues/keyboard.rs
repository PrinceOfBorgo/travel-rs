//! Shared inline-keyboard layout and callback-handling utilities for
//! interactive dialogues.

use crate::{
    Context,
    dialogues::pending_command_dialogue::PendingCommandDialogue,
    i18n::{self, Translate, TranslateWithArgs},
    traveler::Traveler,
};
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::Bot;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message};
use teloxide::{requests::Requester, types::CallbackQuery};

/// Builds an [`InlineKeyboardMarkup`] laid out with `per_row` columns.
///
/// The `buttons` are placed left-to-right, top-to-bottom. The `cancel_button`
/// is always placed in the **last position** of the final row. If the total
/// number of items (buttons + 1 for cancel) is not a multiple of `per_row`,
/// blank spacer buttons are inserted between the last real button and the
/// cancel button so the final row is full-width and every button in the grid
/// has a consistent size.
///
/// `noop_callback` is the callback data attached to spacer buttons. It should
/// use the dialogue's prefix so the callback filter routes it to the handler
/// (which will answer the query and silently ignore it).
///
/// ## Example (per_row = 2, 4 items)
///
/// ```text
/// [item1] [item2]
/// [item3] [item4]
/// [   cancel    ]
/// ```
///
/// ## Example (per_row = 3, 4 items)
///
/// ```text
/// [item1] [item2] [item3]
/// [item4] [ --- ] [ --- ]
/// [       cancel        ]
/// ```
pub fn grid_keyboard(
    buttons: Vec<InlineKeyboardButton>,
    cancel_button: InlineKeyboardButton,
    per_row: usize,
    noop_callback: &str,
) -> InlineKeyboardMarkup {
    let buttons_in_last_row = buttons.len() % per_row;
    let blanks_needed = (per_row - buttons_in_last_row) % per_row; // 0 if last row is already full, otherwise the number of spacers needed

    let blank = || {
        InlineKeyboardButton::callback(
            "\u{2800}".to_owned(), // Braille Pattern Blank — invisible but non-empty
            noop_callback.to_owned(),
        )
    };

    let mut all: Vec<InlineKeyboardButton> = buttons;
    all.extend(std::iter::repeat_with(blank).take(blanks_needed));
    all.push(cancel_button); // Cancel button will always fall in its own row

    let rows: Vec<Vec<InlineKeyboardButton>> = all
        .chunks(per_row)
        .map(<[InlineKeyboardButton]>::to_vec)
        .collect();

    InlineKeyboardMarkup::new(rows)
}

/// Outcome of the common callback pre-processing performed by
/// [`handle_callback_prelude`].
pub enum CallbackAction {
    /// The callback carried a meaningful selection; the stripped value (after
    /// the dialogue prefix) and the original message are returned for further
    /// processing.
    Selection { value: String, msg: Box<Message> },
    /// The callback was fully handled (noop, cancel, or inaccessible
    /// message) — the caller should just return `Ok(())`.
    Handled,
}

/// Configuration bundle for [`handle_callback_prelude`], grouping the
/// dialogue-specific constants that identify special callback data values.
pub struct CallbackConfig<'a> {
    /// The full cancel sentinel (e.g. `example:__cancel__`).
    pub cancel_callback: &'a str,
    /// The full noop sentinel (e.g. `example:__noop__`).
    pub noop_callback: &'a str,
    /// The dialogue callback prefix (e.g. `example:`).
    pub prefix: &'a str,
    /// The i18n key for the process label shown in the cancel confirmation.
    pub running_process_key: &'a str,
}

/// Performs the boilerplate shared by every inline-keyboard callback handler:
///
/// 1. Answers the callback query (dismisses the Telegram spinner).
/// 2. Extracts the original message (bails if inaccessible).
/// 3. Ignores noop (spacer) callbacks.
/// 4. Handles the cancel callback: removes the keyboard, sends the cancel
///    confirmation, exits the dialogue.
/// 5. Strips the prefix from the callback data and returns the remainder
///    as [`CallbackAction::Selection`].
pub async fn handle_callback_prelude(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    q: &CallbackQuery,
    ctx: &Arc<Mutex<Context>>,
    config: &CallbackConfig<'_>,
) -> Result<CallbackAction, Box<dyn std::error::Error + Send + Sync>> {
    // Always answer the callback query so Telegram dismisses the loading
    // spinner on the user's client, regardless of what happens next.
    let _ = bot.answer_callback_query(q.id.clone()).await;

    // Extract the original message. If the message is inaccessible (deleted
    // or older than 48h), there's nothing meaningful we can do — just bail.
    let Some(msg) = q.regular_message().cloned() else {
        tracing::warn!("Callback query without an accessible message; ignoring");
        return Ok(CallbackAction::Handled);
    };

    let data = q.data.as_deref().unwrap_or("");

    // Spacer buttons — do nothing.
    if data == config.noop_callback {
        return Ok(CallbackAction::Handled);
    }

    // Cancel button — clean up and exit the dialogue.
    if data == config.cancel_callback {
        let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;
        let process_name = config.running_process_key.translate(Arc::clone(ctx));
        let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
            ctx.clone(),
            &maplit::hashmap! { i18n::args::PROCESS.into() => process_name.into() },
        );
        bot.send_message(msg.chat.id, cancel_msg).await?;
        dialogue.exit().await?;
        return Ok(CallbackAction::Handled);
    }

    // Strip the prefix to get the actual value selected by the user.
    let value = data.strip_prefix(config.prefix).unwrap_or("").to_owned();
    if value.is_empty() {
        tracing::warn!("Empty value in callback data: {data:?}");
        return Ok(CallbackAction::Handled);
    }

    Ok(CallbackAction::Selection {
        value,
        msg: Box::new(msg),
    })
}

// ─── Traveler-picker keyboard ────────────────────────────────────────────────

/// Number of traveler buttons per row in the inline keyboard.
const TRAVELERS_PER_ROW: usize = 2;

/// Loads the chat's travelers and builds a [`grid_keyboard`] with one button
/// per name (using `prefix` as the callback-data prefix) plus a cancel row.
///
/// Returns `None` if no travelers exist or if the DB query fails.
pub async fn travelers_keyboard(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    prefix: &str,
    cancel_callback: &str,
    noop_callback: &str,
    ctx: Arc<Mutex<Context>>,
) -> Option<InlineKeyboardMarkup> {
    let travelers = Traveler::db_select(db, chat_id).await.ok()?;
    if travelers.is_empty() {
        return None;
    }
    let buttons: Vec<InlineKeyboardButton> = travelers
        .into_iter()
        .map(|t| {
            let name = t.name.to_string();
            InlineKeyboardButton::callback(name.clone(), format!("{prefix}{name}"))
        })
        .collect();

    let cancel_button = InlineKeyboardButton::callback(
        i18n::labels::CANCEL_BUTTON.translate(ctx),
        cancel_callback.to_owned(),
    );

    Some(grid_keyboard(
        buttons,
        cancel_button,
        TRAVELERS_PER_ROW,
        noop_callback,
    ))
}
