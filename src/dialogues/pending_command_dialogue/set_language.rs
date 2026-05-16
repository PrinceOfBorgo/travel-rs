//! `/setlanguage` dialogue: asks the user for the language identifier when
//! the command is invoked without an inline argument.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TryTranslate},
    keyboard::{self, DEFAULT_ROWS_PER_PAGE, PaginatedKeyboardConfig, PickerItem},
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
    types::{CallbackQuery, InlineKeyboardMarkup, Message},
};
use tracing::Level;
use unic_langid::LanguageIdentifier;

// Prefix used to identify callback queries originating from the
// `/setlanguage` inline keyboard.
callback_consts!("setlang" => cancel, noop);

/// Number of language buttons per row in the inline keyboard.
const LANGS_PER_ROW: usize = 2;

#[derive(Debug, Clone)]
pub enum SetLanguageState {
    AskLangid,
}

/// Builds an inline keyboard with the available languages in a uniform grid.
fn available_langs_keyboard(ctx: Arc<Mutex<Context>>) -> InlineKeyboardMarkup {
    let mut items: Vec<PickerItem> = i18n::available_langs()
        .map(|langid| {
            let langid_str = langid.to_string();
            let label = format!("{}{langid_str}", i18n::labels::LANGUAGE_LABEL_PREFIX)
                .try_translate(ctx.clone())
                .unwrap_or_else(|| langid_str.clone());
            PickerItem {
                label,
                value: langid_str,
            }
        })
        .collect();
    items.sort_by(|a, b| a.label.cmp(&b.label));

    // Languages list is small — always fits on one page.
    keyboard::paginated_keyboard(PaginatedKeyboardConfig {
        items: &items,
        page: 0,
        columns: LANGS_PER_ROW,
        rows_per_page: DEFAULT_ROWS_PER_PAGE,
        prefix: CALLBACK_PREFIX,
        cancel_callback: CANCEL_CALLBACK,
        noop_callback: NOOP_CALLBACK,
        action_buttons: &[],
        show_cancel: true,
        ctx,
    })
    .expect("at least one language must be available")
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
    tracing::info!("Dialogue started: /setlanguage");
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

#[apply(trace_callback)]
pub async fn receive_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let action = keyboard::handle_callback_prelude(
        &bot,
        &dialogue,
        &q,
        &ctx,
        &keyboard::CallbackConfig {
            cancel_callback: CANCEL_CALLBACK,
            noop_callback: NOOP_CALLBACK,
            prefix: CALLBACK_PREFIX,
            running_process_key: i18n::commands::RUNNING_PROCESS_SET_LANGUAGE,
        },
    )
    .await?;

    let keyboard::CallbackAction::Selection { value: raw, msg } = action else {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    };

    let Ok(langid) = LanguageIdentifier::from_str(&raw) else {
        tracing::warn!("Invalid langid in callback data: {raw:?}");
        return Ok(());
    };

    keyboard::echo_callback_selection(&bot, &msg, &raw).await;

    let cmd = Command::SetLanguage {
        langid: CommandArg::Provided(langid),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx.clone()).await;

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
