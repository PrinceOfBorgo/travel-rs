//! `/deletetransfer` dialogue: asks the user for the transfer number when
//! the command is invoked without an inline argument. Shows a paginated
//! inline keyboard with the chat's transfers for quick selection; free-text
//! input is accepted as a fallback.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate},
    keyboard::{
        self, CallbackConfig, DEFAULT_ROWS_PER_PAGE, PaginatedCallbackAction,
        PaginatedKeyboardConfig, PickerItem,
    },
    money_wrapper::MoneyWrapper,
    transfer::Transfer,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::{EditMessageReplyMarkupSetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, Message},
};
use tracing::Level;

/// Number of transfer buttons per row in the inline keyboard.
const TRANSFERS_PER_ROW: usize = 1;

// ─── Callback constants ──────────────────────────────────────────────────────

pub const CALLBACK_PREFIX: &str = "delxfr:";
pub const CANCEL_CALLBACK: &str = "delxfr:__cancel__";
const NOOP_CALLBACK: &str = "delxfr:__noop__";

// ─── State ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum DeleteTransferState {
    AskNumber,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn transfer_picker_items(transfers: &[Transfer], ctx: Arc<Mutex<Context>>) -> Vec<PickerItem> {
    transfers
        .iter()
        .map(|t| {
            let amount = MoneyWrapper::new_with_context(t.amount, ctx.clone());
            PickerItem {
                label: format!(
                    "#{}: {} → {} - {}",
                    t.number, t.sender_name, t.receiver_name, amount
                ),
                value: t.number.to_string(),
            }
        })
        .collect()
}

async fn send_prompt_with_keyboard(
    db: Arc<Surreal<Any>>,
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    prompt: String,
    page: usize,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = bot.send_message(chat_id, prompt);
    if let Ok(transfers) = Transfer::transfers(db, chat_id).await {
        let items = transfer_picker_items(&transfers, ctx.clone());
        if let Some(kb) = keyboard::paginated_keyboard(PaginatedKeyboardConfig {
            items: &items,
            page,
            columns: TRANSFERS_PER_ROW,
            rows_per_page: DEFAULT_ROWS_PER_PAGE,
            prefix: CALLBACK_PREFIX,
            cancel_callback: CANCEL_CALLBACK,
            noop_callback: NOOP_CALLBACK,
            action_buttons: &[],
            show_cancel: true,
            ctx,
        }) {
            request = request.reply_markup(kb);
        }
    }
    request.await?;
    Ok(())
}

// ─── Start ───────────────────────────────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn start(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let prompt = i18n::dialogues::DELETE_TRANSFER_ASK_NUMBER.translate(ctx.clone());
    send_prompt_with_keyboard(db, &bot, msg.chat.id, prompt, 0, ctx).await?;
    dialogue
        .update(PendingCommandState::DeleteTransfer(
            DeleteTransferState::AskNumber,
        ))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── Text handler ────────────────────────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn receive_number(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    let number = match text.parse::<i64>() {
        Ok(n) => n,
        Err(_) => {
            tracing::warn!("Invalid transfer number: {text:?}");
            bot.send_message(
                msg.chat.id,
                i18n::dialogues::DELETE_TRANSFER_INVALID_NUMBER.translate(ctx),
            )
            .await?;
            return Ok(());
        }
    };

    let cmd = Command::DeleteTransfer {
        number: CommandArg::Provided(number),
    };
    let outcome = command_reply(db.clone(), &msg, &cmd, ctx.clone()).await;
    bot.send_message(msg.chat.id, outcome.message()).await?;
    if outcome.is_success() {
        dialogue.exit().await?;
    } else {
        let prompt = i18n::dialogues::DELETE_TRANSFER_ASK_NUMBER.translate(ctx.clone());
        send_prompt_with_keyboard(db, &bot, msg.chat.id, prompt, 0, ctx).await?;
    }
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── Callback handler ────────────────────────────────────────────────────────

#[apply(trace_callback)]
pub async fn receive_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let action = keyboard::handle_paginated_callback(
        &bot,
        &dialogue,
        &q,
        &ctx,
        &CallbackConfig {
            cancel_callback: CANCEL_CALLBACK,
            noop_callback: NOOP_CALLBACK,
            prefix: CALLBACK_PREFIX,
            running_process_key: i18n::commands::RUNNING_PROCESS_DELETE_TRANSFER,
        },
    )
    .await?;

    match action {
        PaginatedCallbackAction::Selection { value, msg } => {
            let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;
            let Ok(number) = value.parse::<i64>() else {
                tracing::warn!("Invalid number in callback data: {value:?}");
                return Ok(());
            };
            let cmd = Command::DeleteTransfer {
                number: CommandArg::Provided(number),
            };
            let fake_msg = msg.as_ref();
            let outcome = command_reply(db.clone(), fake_msg, &cmd, ctx.clone()).await;
            bot.send_message(msg.chat.id, outcome.message()).await?;
            if outcome.is_success() {
                dialogue.exit().await?;
            } else {
                let prompt = i18n::dialogues::DELETE_TRANSFER_ASK_NUMBER.translate(ctx.clone());
                send_prompt_with_keyboard(db, &bot, msg.chat.id, prompt, 0, ctx).await?;
            }
        }
        PaginatedCallbackAction::PageChange { page, msg } => {
            if let Ok(transfers) = Transfer::transfers(db, msg.chat.id).await {
                let items = transfer_picker_items(&transfers, ctx.clone());
                if let Some(kb) = keyboard::paginated_keyboard(PaginatedKeyboardConfig {
                    items: &items,
                    page,
                    columns: TRANSFERS_PER_ROW,
                    rows_per_page: DEFAULT_ROWS_PER_PAGE,
                    prefix: CALLBACK_PREFIX,
                    cancel_callback: CANCEL_CALLBACK,
                    noop_callback: NOOP_CALLBACK,
                    action_buttons: &[],
                    show_cancel: true,
                    ctx,
                }) {
                    let _ = bot
                        .edit_message_reply_markup(msg.chat.id, msg.id)
                        .reply_markup(kb)
                        .await;
                }
            }
        }
        PaginatedCallbackAction::Handled => {}
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

    test! { ask_number_on_empty_invocation,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer");
        let response = i18n::dialogues::DELETE_TRANSFER_ASK_NUMBER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_number_invalid_reprompts,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer");
        bot.dispatch().await;

        bot.update("not a number");
        let response = i18n::dialogues::DELETE_TRANSFER_INVALID_NUMBER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_number_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer");
        bot.dispatch().await;

        // After a not-found reply, the dialogue re-prompts so the user can retry.
        bot.update("999");
        let response = i18n::dialogues::DELETE_TRANSFER_ASK_NUMBER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { cancel_during_dialogue,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_DELETE_TRANSFER);
        bot.test_last_message(&response).await;
    }
}
