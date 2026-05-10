//! `/deletetraveler` dialogue: asks the user for the traveler's name when
//! the command is invoked without an inline argument, then delegates to the
//! regular command handler. Shows an inline keyboard with the chat's
//! travelers for quick selection; free-text input is still accepted.
//! A confirmation step is shown before the actual deletion.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TranslateWithArgs},
    keyboard::{self, ConfirmAnswer, ConfirmConfig, confirmation_keyboard, parse_confirm_answer},
    traveler::Name,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, Message},
};
use tracing::Level;

/// Prefix used to identify callback queries originating from the
/// `/deletetraveler` inline keyboard. The remainder of the callback data is
/// the selected traveler name (e.g. `deltrav:Alice`).
pub const CALLBACK_PREFIX: &str = "deltrav:";

/// Callback data used by the cancel button.
pub const CANCEL_CALLBACK_DATA: &str = "deltrav:__cancel__";

/// Callback data for blank spacer buttons.
const NOOP_CALLBACK_DATA: &str = "deltrav:__noop__";

pub const CONFIRM_CALLBACK: &str = "deltrav:__confirm__";
pub const DENY_CALLBACK: &str = "deltrav:__deny__";

#[derive(Debug, Clone)]
pub enum DeleteTravelerState {
    AskName,
    Confirm(Name),
}

/// Sends the confirmation prompt with a Yes / No keyboard and transitions the
/// dialogue into the [`DeleteTravelerState::Confirm`] state.
async fn ask_confirmation(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let prompt = i18n::dialogues::DELETE_TRAVELER_CONFIRM.translate_with_args(
        ctx.clone(),
        &hashmap! { i18n::args::NAME.into() => name.to_string().into() },
    );
    let kb = confirmation_keyboard(ConfirmConfig {
        confirm_callback: CONFIRM_CALLBACK,
        deny_callback: DENY_CALLBACK,
        ctx,
    });
    bot.send_message(chat_id, prompt).reply_markup(kb).await?;
    dialogue
        .update(PendingCommandState::DeleteTraveler(
            DeleteTravelerState::Confirm(name),
        ))
        .await?;
    Ok(())
}

#[apply(trace_state_db)]
pub async fn start(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let prompt = i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate(ctx.clone());
    let mut request = bot.send_message(msg.chat.id, prompt);
    if let Some(kb) = keyboard::travelers_keyboard(keyboard::TravelersKeyboardConfig {
        db,
        chat_id: msg.chat.id,
        prefix: CALLBACK_PREFIX,
        cancel_callback: CANCEL_CALLBACK_DATA,
        noop_callback: NOOP_CALLBACK_DATA,
        show_cancel: true,
        ctx,
    })
    .await
    {
        request = request.reply_markup(kb);
    }
    request.await?;
    dialogue
        .update(PendingCommandState::DeleteTraveler(
            DeleteTravelerState::AskName,
        ))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    tracing::info!("Dialogue started: /deletetraveler");
    Ok(())
}

#[apply(trace_state)]
pub async fn receive_name(
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    if text.is_empty() {
        tracing::warn!("Invalid name: received empty input.");
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::DELETE_TRAVELER_INVALID_NAME.translate(ctx),
        )
        .await?;
        return Ok(());
    }

    let name = match Name::from_str(text) {
        Ok(name) => name,
        Err(err) => {
            tracing::warn!("{err}");
            let reply = format!(
                "{invalid}\n\n{reason}",
                invalid = i18n::dialogues::DELETE_TRAVELER_INVALID_NAME.translate(ctx.clone()),
                reason = err.translate(ctx),
            );
            bot.send_message(msg.chat.id, reply).await?;
            return Ok(());
        }
    };

    ask_confirmation(&bot, &dialogue, msg.chat.id, name, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[apply(trace_callback)]
pub async fn receive_callback(
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
            cancel_callback: CANCEL_CALLBACK_DATA,
            noop_callback: NOOP_CALLBACK_DATA,
            prefix: CALLBACK_PREFIX,
            running_process_key: i18n::commands::RUNNING_PROCESS_DELETE_TRAVELER,
        },
    )
    .await?;

    let keyboard::CallbackAction::Selection { value: raw, msg } = action else {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    };

    let Ok(name) = Name::from_str(&raw) else {
        tracing::warn!("Invalid name in callback data: {raw:?}");
        return Ok(());
    };

    // Remove the inline keyboard.
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    ask_confirmation(&bot, &dialogue, msg.chat.id, name, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── Start (inline form with pre-supplied name) ──────────────────────────────

/// Entry point for the inline form (`/deletetraveler Alice`). Skips the name
/// prompt and jumps straight to the confirmation step.
#[apply(trace_state)]
pub async fn start_confirm(
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    name: CommandArg<Name>,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let name = name.expect_provided("deletetraveler");
    ask_confirmation(&bot, &dialogue, msg.chat.id, name.clone(), ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    tracing::info!("Dialogue started: /deletetraveler (inline confirm '{name}')");
    Ok(())
}

// ─── Confirm callback handler ────────────────────────────────────────────────

/// Text handler for the Confirm state — accepts yes/no/y/n keywords.
#[apply(trace_state_db)]
pub async fn receive_confirm_text(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    match parse_confirm_answer(text) {
        ConfirmAnswer::Yes => {
            let cmd = Command::DeleteTraveler {
                name: CommandArg::Provided(name),
            };
            let outcome = command_reply(db, &msg, &cmd, ctx).await;
            bot.send_message(msg.chat.id, outcome.message()).await?;
            dialogue.exit().await?;
        }
        ConfirmAnswer::No => {
            dialogue.exit().await?;
            let process_name =
                i18n::commands::RUNNING_PROCESS_DELETE_TRAVELER.translate(Arc::clone(&ctx));
            let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
                ctx,
                &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
            );
            bot.send_message(msg.chat.id, cancel_msg).await?;
        }
        ConfirmAnswer::Unknown => {
            ask_confirmation(&bot, &dialogue, msg.chat.id, name, ctx).await?;
        }
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[apply(trace_callback)]
pub async fn receive_confirm_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let _ = bot.answer_callback_query(q.id.clone()).await;

    let Some(msg) = q.regular_message().cloned() else {
        tracing::warn!("Callback query without an accessible message; ignoring");
        return Ok(());
    };

    let data = q.data.as_deref().unwrap_or("");

    // Remove the confirmation keyboard.
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    if data == CONFIRM_CALLBACK {
        let cmd = Command::DeleteTraveler {
            name: CommandArg::Provided(name),
        };
        let outcome = command_reply(db, &msg, &cmd, ctx).await;
        bot.send_message(msg.chat.id, outcome.message()).await?;
        dialogue.exit().await?;
    } else {
        // Deny or unexpected data → cancel.
        dialogue.exit().await?;
        let process_name =
            i18n::commands::RUNNING_PROCESS_DELETE_TRAVELER.translate(Arc::clone(&ctx));
        let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
            ctx,
            &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
        );
        bot.send_message(msg.chat.id, cancel_msg).await?;
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        errors::NameValidationError,
        i18n::{self, Translate, TranslateWithArgs},
        tests::{TestBot, helpers},
    };
    use maplit::hashmap;

    test! { ask_name_on_empty_invocation,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetraveler");
        let response = i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_name_ok,
        let db = db().await;

        // Pre-add Alice using the inline form.
        let mut bot = TestBot::new(db, "");
        helpers::add_traveler(&mut bot, "Alice").await;

        // Now invoke /deletetraveler without a name.
        bot.update("/deletetraveler");
        bot.dispatch().await;

        // Typing the name transitions to the confirmation step.
        bot.update("Alice");
        let response = i18n::dialogues::DELETE_TRAVELER_CONFIRM.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { receive_name_trims_whitespace,
        let db = db().await;

        let mut bot = TestBot::new(db, "");
        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/deletetraveler");
        bot.dispatch().await;

        bot.update("   Alice   ");
        let response = i18n::dialogues::DELETE_TRAVELER_CONFIRM.translate_with_args_default(
            &hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { receive_name_empty_input_reprompts,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetraveler");
        bot.dispatch().await;

        bot.update("   ");
        let response = i18n::dialogues::DELETE_TRAVELER_INVALID_NAME.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_name_invalid_name_reprompts,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetraveler");
        bot.dispatch().await;

        // A name starting with `/` is invalid (looks like a command).
        bot.update("/Alice");
        let invalid = i18n::dialogues::DELETE_TRAVELER_INVALID_NAME.translate_default();
        let reason = NameValidationError::StartsWithSlash("/Alice".to_owned()).translate_default();
        let expected = format!("{invalid}\n\n{reason}");
        bot.test_last_message(&expected).await;
    }

    test! { dialogue_stays_alive_after_confirmation,
        let db = db().await;

        // Add traveler "Alice" so the name is valid.
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        bot.update("/deletetraveler");
        bot.dispatch().await;
        // Typing name transitions to Confirm state.
        bot.update("Alice");
        bot.dispatch().await;

        // Dialogue is still alive (waiting for callback confirmation).
        // /cancel acknowledges the still-running dialogue.
        bot.update("/cancel");
        let response = crate::tests::helpers::cancel_ok_for(
            i18n::commands::RUNNING_PROCESS_DELETE_TRAVELER,
        );
        bot.test_last_message(&response).await;
    }

    test! { cancel_during_dialogue,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetraveler");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = crate::tests::helpers::cancel_ok_for(
            i18n::commands::RUNNING_PROCESS_DELETE_TRAVELER,
        );
        bot.test_last_message(&response).await;
    }
}
