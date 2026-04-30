//! `/setlanguage` dialogue: asks the user for the language identifier when
//! the command is invoked without an inline argument.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TranslateWithArgs, TryTranslate},
    trace_state, trace_state_db,
};
use macro_rules_attribute::apply;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use tracing::Level;
use unic_langid::LanguageIdentifier;

/// Prefix used to identify callback queries originating from the
/// `/setlanguage` inline keyboard. The remainder of the callback data is the
/// selected language identifier (e.g. `setlang:it-IT`).
pub const CALLBACK_PREFIX: &str = "setlang:";

/// Callback data used by the cancel button in the `/setlanguage` inline
/// keyboard.
pub const CANCEL_CALLBACK_DATA: &str = "setlang:__cancel__";

/// Number of language buttons per row in the inline keyboard.
const LANGS_PER_ROW: usize = 3;

#[derive(Debug, Clone)]
pub enum SetLanguageState {
    AskLangid,
}

/// Builds an inline keyboard with the available languages laid out in a
/// compact grid plus a localized cancel button on its own row.
fn available_langs_keyboard(ctx: Arc<Mutex<Context>>) -> InlineKeyboardMarkup {
    let mut lang_buttons: Vec<InlineKeyboardButton> = i18n::available_langs()
        .map(|langid| {
            let langid_str = langid.to_string();
            // Fall back to langid when the label is not defined
            let label = format!("{}{langid_str}", i18n::labels::LANGUAGE_LABEL_PREFIX)
                .try_translate(ctx.clone())
                .unwrap_or_else(|| langid_str.clone());
            InlineKeyboardButton::callback(label, format!("{CALLBACK_PREFIX}{langid_str}"))
        })
        .collect();
    lang_buttons.sort_by(|a, b| a.text.cmp(&b.text));

    let mut rows: Vec<Vec<InlineKeyboardButton>> = lang_buttons
        .chunks(LANGS_PER_ROW)
        .map(<[InlineKeyboardButton]>::to_vec)
        .collect();

    rows.push(vec![InlineKeyboardButton::callback(
        i18n::labels::CANCEL_BUTTON.translate(ctx),
        CANCEL_CALLBACK_DATA,
    )]);

    InlineKeyboardMarkup::new(rows)
}

#[apply(trace_state)]
pub async fn start(
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    bot.send_message(
        msg.chat.id,
        i18n::dialogues::SET_LANGUAGE_ASK_LANGID.translate(ctx.clone()),
    )
    .reply_markup(available_langs_keyboard(ctx))
    .await?;
    dialogue
        .update(PendingCommandState::SetLanguage(
            SetLanguageState::AskLangid,
        ))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[apply(trace_state_db)]
pub async fn receive_langid(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    let langid = match LanguageIdentifier::from_str(text) {
        Ok(l) => l,
        Err(_) => {
            tracing::warn!("Invalid langid: {text:?}");
            bot.send_message(
                msg.chat.id,
                i18n::dialogues::SET_LANGUAGE_INVALID_LANGID.translate(ctx),
            )
            .await?;
            return Ok(());
        }
    };

    let cmd = Command::SetLanguage {
        langid: CommandArg::Provided(langid),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx.clone()).await;
    bot.send_message(msg.chat.id, outcome.message()).await?;
    if outcome.is_success() {
        dialogue.exit().await?;
    } else {
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::SET_LANGUAGE_ASK_LANGID.translate(ctx.clone()),
        )
        .reply_markup(available_langs_keyboard(ctx))
        .await?;
    }
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[tracing::instrument(
    err(level = Level::ERROR),
    ret(level = Level::DEBUG),
    skip_all,
    fields(chat_id = ?q.regular_message().map(|m| m.chat.id), sender_id = %q.from.id),
)]
pub async fn receive_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    // Always answer the callback query so Telegram dismisses the loading
    // spinner on the user's client, regardless of what happens next.
    let _ = bot.answer_callback_query(q.id.clone()).await;

    // Extract the original message (needed by `command_reply` and to remove
    // the inline keyboard). If the message is inaccessible (deleted or older
    // than 48h), there's nothing meaningful we can do — just bail.
    let Some(msg) = q.regular_message().cloned() else {
        tracing::warn!("Callback query without an accessible message; ignoring");
        return Ok(());
    };

    // Parse the callback data: it's either the cancel sentinel or a langid
    // prefixed with `CALLBACK_PREFIX`.
    let data = q.data.as_deref().unwrap_or("");

    if data == CANCEL_CALLBACK_DATA {
        // Remove the inline keyboard from the original prompt (best-effort).
        let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;
        let process_name =
            i18n::commands::RUNNING_PROCESS_SET_LANGUAGE.translate(Arc::clone(&ctx));
        let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
            ctx,
            &maplit::hashmap! { i18n::args::PROCESS.into() => process_name.into() },
        );
        bot.send_message(msg.chat.id, cancel_msg).await?;
        dialogue.exit().await?;
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    }

    let raw = data.strip_prefix(CALLBACK_PREFIX).unwrap_or("");
    let Ok(langid) = LanguageIdentifier::from_str(raw) else {
        tracing::warn!("Invalid langid in callback data: {raw:?}");
        return Ok(());
    };

    let cmd = Command::SetLanguage {
        langid: CommandArg::Provided(langid),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx.clone()).await;

    // Remove the inline keyboard from the original prompt so the user
    // can no longer interact with it. Best-effort: ignore errors (e.g. if
    // the message was deleted).
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    bot.send_message(msg.chat.id, outcome.message()).await?;
    if outcome.is_success() {
        dialogue.exit().await?;
    } else {
        // Dialogue stays alive: re-send the prompt with a fresh keyboard
        // so the user knows they can retry.
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::SET_LANGUAGE_ASK_LANGID.translate(ctx.clone()),
        )
        .reply_markup(available_langs_keyboard(ctx))
        .await?;
    }
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate},
        tests::{TestBot, helpers::cancel_ok_for},
    };

    test! { ask_langid_on_empty_invocation,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setlanguage");
        let response = i18n::dialogues::SET_LANGUAGE_ASK_LANGID.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_langid_invalid_reprompts,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setlanguage");
        bot.dispatch().await;

        bot.update("not-a-langid!!");
        let response = i18n::dialogues::SET_LANGUAGE_INVALID_LANGID.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_langid_unavailable_reprompts,
        let db = db().await;

        // Pick a well-formed langid that is not bundled with the bot.
        let unavailable = "ab-CD";

        let mut bot = TestBot::new(db, "/setlanguage");
        bot.dispatch().await;

        bot.update(unavailable);
        // After the not-available reply, the dialogue re-prompts the user.
        let response = i18n::dialogues::SET_LANGUAGE_ASK_LANGID.translate_default();
        bot.test_last_message(&response).await;

        // Dialogue is still alive: /cancel acknowledges instead of going
        // through the unknown-command path.
        bot.update("/cancel");
        let cancel = cancel_ok_for(i18n::commands::RUNNING_PROCESS_SET_LANGUAGE);
        bot.test_last_message(&cancel).await;
    }

    test! { cancel_during_dialogue,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setlanguage");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_SET_LANGUAGE);
        bot.test_last_message(&response).await;
    }
}
