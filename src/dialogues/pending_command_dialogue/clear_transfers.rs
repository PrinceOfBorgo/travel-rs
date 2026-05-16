//! `/cleartransfers` confirmation dialogue.

use crate::{
    Context, HandlerResult,
    commands::{Command, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TranslateWithArgs},
    keyboard::{self, ConfirmAnswer, ConfirmConfig, confirmation_keyboard, parse_confirm_answer},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, Message},
};
use tracing::Level;

// ─── Callback constants ──────────────────────────────────────────────────────

callback_consts!("clrxfr" => confirm, deny);

// ─── State ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ClearTransfersState {
    Confirm,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn ask_confirmation(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let prompt = i18n::dialogues::CLEAR_TRANSFERS_CONFIRM.translate(ctx.clone());
    let kb = confirmation_keyboard(ConfirmConfig {
        confirm_callback: CONFIRM_CALLBACK,
        deny_callback: DENY_CALLBACK,
        ctx,
    });
    bot.send_message(chat_id, prompt).reply_markup(kb).await?;
    dialogue
        .update(PendingCommandState::ClearTransfers(
            ClearTransfersState::Confirm,
        ))
        .await?;
    Ok(())
}

// ─── Start ───────────────────────────────────────────────────────────────────

#[apply(trace_state)]
pub async fn start(
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    ask_confirmation(&bot, &dialogue, msg.chat.id, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    tracing::info!("Dialogue started: /cleartransfers");
    Ok(())
}

// ─── Confirm text handler ────────────────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn receive_confirm_text(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    match parse_confirm_answer(text) {
        ConfirmAnswer::Yes => {
            let cmd = Command::ClearTransfers;
            let outcome = command_reply(db, &msg, &cmd, ctx).await;
            bot.send_message(msg.chat.id, outcome.message()).await?;
            dialogue.exit().await?;
        }
        ConfirmAnswer::No => {
            dialogue.exit().await?;
            let process_name =
                i18n::commands::RUNNING_PROCESS_CLEAR_TRANSFERS.translate(Arc::clone(&ctx));
            let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
                ctx,
                &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
            );
            bot.send_message(msg.chat.id, cancel_msg).await?;
        }
        ConfirmAnswer::Unknown => {
            ask_confirmation(&bot, &dialogue, msg.chat.id, ctx).await?;
        }
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── Confirm callback handler ────────────────────────────────────────────────

#[apply(trace_callback)]
pub async fn receive_confirm_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let _ = bot.answer_callback_query(q.id.clone()).await;

    let Some(msg) = q.regular_message().cloned() else {
        tracing::warn!("Callback query without an accessible message; ignoring");
        return Ok(());
    };

    let data = q.data.as_deref().unwrap_or("");

    let label = if data == CONFIRM_CALLBACK {
        i18n::labels::CONFIRM_YES_BUTTON.translate(ctx.clone())
    } else {
        i18n::labels::CONFIRM_NO_BUTTON.translate(ctx.clone())
    };
    keyboard::echo_callback_selection(&bot, &msg, &label).await;

    if data == CONFIRM_CALLBACK {
        let cmd = Command::ClearTransfers;
        let outcome = command_reply(db, &msg, &cmd, ctx).await;
        bot.send_message(msg.chat.id, outcome.message()).await?;
        dialogue.exit().await?;
    } else {
        dialogue.exit().await?;
        let process_name =
            i18n::commands::RUNNING_PROCESS_CLEAR_TRANSFERS.translate(Arc::clone(&ctx));
        let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
            ctx,
            &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
        );
        bot.send_message(msg.chat.id, cancel_msg).await?;
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}
