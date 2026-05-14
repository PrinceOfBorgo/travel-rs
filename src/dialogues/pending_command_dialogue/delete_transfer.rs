//! `/deletetransfer` dialogue: asks the user for the transfer number when
//! the command is invoked without an inline argument. Shows a paginated
//! inline keyboard with the chat's transfers for quick selection; free-text
//! input is accepted as a fallback. A confirmation step is shown before the
//! actual deletion.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TranslateWithArgs},
    keyboard::{
        self, CallbackConfig, ConfirmAnswer, ConfirmConfig, DEFAULT_ROWS_PER_PAGE,
        PaginatedCallbackAction, PaginatedKeyboardConfig, PickerItem, confirmation_keyboard,
        parse_confirm_answer,
    },
    money_wrapper::MoneyWrapper,
    transfer::Transfer,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
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
pub const CONFIRM_CALLBACK: &str = "delxfr:__confirm__";
pub const DENY_CALLBACK: &str = "delxfr:__deny__";

// ─── State ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum DeleteTransferState {
    AskNumber,
    Confirm(i64),
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

/// Sends the confirmation prompt with a Yes / No keyboard and transitions the
/// dialogue into the [`DeleteTransferState::Confirm`] state.
async fn ask_confirmation(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let prompt = i18n::dialogues::DELETE_TRANSFER_CONFIRM.translate_with_args(
        ctx.clone(),
        &hashmap! { i18n::args::NUMBER.into() => number.into() },
    );
    let kb = confirmation_keyboard(ConfirmConfig {
        confirm_callback: CONFIRM_CALLBACK,
        deny_callback: DENY_CALLBACK,
        ctx,
    });
    bot.send_message(chat_id, prompt).reply_markup(kb).await?;
    dialogue
        .update(PendingCommandState::DeleteTransfer(
            DeleteTransferState::Confirm(number),
        ))
        .await?;
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
    tracing::info!("Dialogue started: /deletetransfer");
    Ok(())
}

// ─── Text handler ────────────────────────────────────────────────────────────

#[apply(trace_state)]
pub async fn receive_number(
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

    ask_confirmation(&bot, &dialogue, msg.chat.id, number, ctx).await?;
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
            ask_confirmation(&bot, &dialogue, msg.chat.id, number, ctx).await?;
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

// ─── Start (inline form with pre-supplied number) ────────────────────────────

/// Entry point for the inline form (`/deletetransfer 3`). Skips the number
/// prompt and jumps straight to the confirmation step.
#[apply(trace_state)]
pub async fn start_confirm(
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    number: CommandArg<i64>,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let number = number.expect_provided("deletetransfer");
    ask_confirmation(&bot, &dialogue, msg.chat.id, number, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    tracing::info!("Dialogue started: /deletetransfer (inline confirm #{number})");
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
    number: i64,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    match parse_confirm_answer(text) {
        ConfirmAnswer::Yes => {
            let cmd = Command::DeleteTransfer {
                number: CommandArg::Provided(number),
            };
            let outcome = command_reply(db, &msg, &cmd, ctx).await;
            bot.send_message(msg.chat.id, outcome.message()).await?;
            dialogue.exit().await?;
        }
        ConfirmAnswer::No => {
            dialogue.exit().await?;
            let process_name =
                i18n::commands::RUNNING_PROCESS_DELETE_TRANSFER.translate(Arc::clone(&ctx));
            let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
                ctx,
                &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
            );
            bot.send_message(msg.chat.id, cancel_msg).await?;
        }
        ConfirmAnswer::Unknown => {
            ask_confirmation(&bot, &dialogue, msg.chat.id, number, ctx).await?;
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
    number: i64,
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
        let cmd = Command::DeleteTransfer {
            number: CommandArg::Provided(number),
        };
        let outcome = command_reply(db, &msg, &cmd, ctx).await;
        bot.send_message(msg.chat.id, outcome.message()).await?;
        dialogue.exit().await?;
    } else {
        // Deny or unexpected data → cancel.
        dialogue.exit().await?;
        let process_name =
            i18n::commands::RUNNING_PROCESS_DELETE_TRANSFER.translate(Arc::clone(&ctx));
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
        i18n::{self, Translate, TranslateWithArgs},
        tests::{TestBot, helpers::cancel_ok_for},
    };
    use maplit::hashmap;

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

    test! { receive_number_shows_confirmation,
        let db = db().await;

        let mut bot = TestBot::new(db, "/deletetransfer");
        bot.dispatch().await;

        // Entering a number transitions to the confirmation step.
        bot.update("999");
        let response = i18n::dialogues::DELETE_TRANSFER_CONFIRM.translate_with_args_default(
            &hashmap! {i18n::args::NUMBER.into() => 999.into()},
        );
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
