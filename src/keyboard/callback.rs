//! Common callback-handling logic for inline-keyboard dialogues.

use crate::{
    Context,
    dialogues::pending_command_dialogue::PendingCommandDialogue,
    i18n::{self, Translate, TranslateWithArgs},
};
use std::sync::{Arc, Mutex};
use teloxide::Bot;
use teloxide::types::Message;
use teloxide::{requests::Requester, types::CallbackQuery};

/// Outcome of the common callback pre-processing performed by
/// [`handle_callback_prelude`].
pub enum CallbackAction {
    /// The callback carried a meaningful selection; the stripped value (after
    /// the dialogue prefix) and the original message are returned for further
    /// processing.
    Selection { value: String, msg: Box<Message> },
    /// The callback was fully handled (noop, cancel, inaccessible message,
    /// or invalid callback data) — the caller should just return `Ok(())`.
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

/// Edits the callback message to append the user's selection with a ✓ prefix
/// and removes the inline keyboard.
///
/// This is the standard visual feedback for inline-keyboard selections:
/// the original prompt is kept, and the chosen option is appended below it.
/// Best-effort: errors (e.g. message already deleted) are silently ignored.
pub async fn echo_callback_selection(bot: &Bot, msg: &Message, label: &str) {
    if let Some(text) = msg.text() {
        let _ = bot
            .edit_message_text(msg.chat.id, msg.id, format!("{text}\n✓ {label}"))
            .await;
    }
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;
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
