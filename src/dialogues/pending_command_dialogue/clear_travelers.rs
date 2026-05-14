//! `/cleartravelers` confirmation dialogue.

use crate::{
    Context, HandlerResult,
    commands::{Command, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    expense::Expense,
    i18n::{self, Translate, TranslateWithArgs},
    keyboard::{self, ConfirmAnswer, ConfirmConfig, confirmation_keyboard, parse_confirm_answer},
    traveler::Traveler,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use tracing::Level;

// ─── Callback constants ──────────────────────────────────────────────────────

pub const CALLBACK_PREFIX: &str = "clrtrav:";
pub const CONFIRM_CALLBACK: &str = "clrtrav:__confirm__";
pub const DENY_CALLBACK: &str = "clrtrav:__deny__";
/// Prefix for "show expenses of traveler" buttons.
pub const SHOW_PREFIX: &str = "clrtrav:show:";
/// Callback for the "All" button.
pub const SHOW_ALL_CALLBACK: &str = "clrtrav:show:__all__";

// ─── State ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ClearTravelersState {
    Confirm,
    /// Waiting for the user to pick a traveler (or "All") to view expenses.
    ShowExpenses,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn ask_confirmation(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let prompt = i18n::dialogues::CLEAR_TRAVELERS_CONFIRM.translate(ctx.clone());
    let kb = confirmation_keyboard(ConfirmConfig {
        confirm_callback: CONFIRM_CALLBACK,
        deny_callback: DENY_CALLBACK,
        ctx,
    });
    bot.send_message(chat_id, prompt).reply_markup(kb).await?;
    dialogue
        .update(PendingCommandState::ClearTravelers(
            ClearTravelersState::Confirm,
        ))
        .await?;
    Ok(())
}

/// Builds an inline keyboard with one button per traveler (in rows of 2)
/// and an "All" button on the last row. Each button uses the traveler's
/// stable `number` field as callback data.
fn show_expenses_keyboard(
    travelers: &[(Traveler, Vec<Expense>)],
    ctx: Arc<Mutex<Context>>,
) -> InlineKeyboardMarkup {
    let buttons: Vec<InlineKeyboardButton> = travelers
        .iter()
        .map(|(t, _)| {
            InlineKeyboardButton::callback(t.name.to_string(), format!("{SHOW_PREFIX}{}", t.number))
        })
        .collect();
    let mut rows: Vec<Vec<InlineKeyboardButton>> = buttons
        .chunks(2)
        .map(<[InlineKeyboardButton]>::to_vec)
        .collect();
    rows.push(vec![InlineKeyboardButton::callback(
        i18n::labels::ALL_BUTTON.translate(ctx),
        SHOW_ALL_CALLBACK,
    )]);
    InlineKeyboardMarkup::new(rows)
}

/// Formats expenses using the same style as `/deletetraveler`'s has-expenses
/// output.
fn format_expenses(expenses: &[Expense], ctx: Arc<Mutex<Context>>) -> String {
    expenses
        .iter()
        .map(|e| e.translate(ctx.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Handles the "yes" confirmation: checks for travelers with expenses and
/// either proceeds with deletion or shows the expense-viewing keyboard.
async fn handle_confirm_yes(
    db: Arc<Surreal<Any>>,
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let with_expenses = Traveler::travelers_with_expenses(db.clone(), chat_id).await;
    match with_expenses {
        Ok(list) if list.len() == 1 => {
            // Single traveler with expenses — show them directly (no keyboard).
            let (traveler, expenses) = &list[0];
            let has_expenses_msg = i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES
                .translate_with_args(
                    ctx.clone(),
                    &hashmap! { i18n::args::TRAVELERS.into() => traveler.name.to_string().into() },
                );
            let expenses_reply = format_expenses(expenses, ctx);
            let reply = format!("{has_expenses_msg}\n\n{expenses_reply}");
            bot.send_message(chat_id, reply).await?;
            dialogue.exit().await?;
        }
        Ok(list) if !list.is_empty() => {
            let names: Vec<String> = list.iter().map(|(t, _)| t.name.to_string()).collect();
            let has_expenses_msg = i18n::commands::CLEAR_TRAVELERS_HAS_EXPENSES
                .translate_with_args(
                    ctx.clone(),
                    &hashmap! { i18n::args::TRAVELERS.into() => names.join("\n").into() },
                );
            let prompt = format!(
                "{has_expenses_msg}\n\n{}",
                i18n::dialogues::CLEAR_TRAVELERS_SHOW_EXPENSES_PROMPT.translate(ctx.clone()),
            );
            let kb = show_expenses_keyboard(&list, ctx);
            bot.send_message(chat_id, prompt).reply_markup(kb).await?;
            dialogue
                .update(PendingCommandState::ClearTravelers(
                    ClearTravelersState::ShowExpenses,
                ))
                .await?;
        }
        Ok(_) => {
            // No travelers with expenses — proceed with deletion.
            let cmd = Command::ClearTravelers;
            let outcome = command_reply(db, msg, &cmd, ctx).await;
            bot.send_message(chat_id, outcome.message()).await?;
            dialogue.exit().await?;
        }
        Err(_) => {
            // Error already logged inside travelers_with_expenses.
            let outcome = command_reply(db, msg, &Command::ClearTravelers, ctx).await;
            bot.send_message(chat_id, outcome.message()).await?;
            dialogue.exit().await?;
        }
    }
    Ok(())
}

fn cancel_message(ctx: Arc<Mutex<Context>>) -> String {
    let process_name = i18n::commands::RUNNING_PROCESS_CLEAR_TRAVELERS.translate(Arc::clone(&ctx));
    i18n::commands::CANCEL_OK.translate_with_args(
        ctx,
        &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
    )
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
    tracing::info!("Dialogue started: /cleartravelers");
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
            handle_confirm_yes(db, &bot, &dialogue, msg.chat.id, &msg, ctx).await?;
        }
        ConfirmAnswer::No => {
            dialogue.exit().await?;
            bot.send_message(msg.chat.id, cancel_message(ctx)).await?;
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
        handle_confirm_yes(db, &bot, &dialogue, msg.chat.id, &msg, ctx).await?;
    } else {
        dialogue.exit().await?;
        bot.send_message(msg.chat.id, cancel_message(ctx)).await?;
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── ShowExpenses callback handler ───────────────────────────────────────────

#[apply(trace_callback)]
pub async fn receive_show_expenses_callback(
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

    // Resolve the label for the selected button.
    let label = if data == SHOW_ALL_CALLBACK {
        i18n::labels::ALL_BUTTON.translate(ctx.clone())
    } else if let Some(num_str) = data.strip_prefix(SHOW_PREFIX) {
        if let Ok(number) = num_str.parse::<i64>() {
            Traveler::db_select_by_number(db.clone(), msg.chat.id, number)
                .await
                .ok()
                .flatten()
                .map(|t| t.name.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    if !label.is_empty() {
        keyboard::echo_callback_selection(&bot, &msg, &label).await;
    }

    if data == SHOW_ALL_CALLBACK {
        // Show expenses for all travelers that have them, grouped by traveler.
        match Traveler::travelers_with_expenses(db, msg.chat.id).await {
            Ok(list) => {
                let sections: Vec<String> = list
                    .iter()
                    .filter(|(_, exps)| !exps.is_empty())
                    .map(|(t, exps)| format!("{}:\n{}", t.name, format_expenses(exps, ctx.clone())))
                    .collect();
                if sections.is_empty() {
                    tracing::info!("No expenses found (race condition or already deleted)");
                } else {
                    let reply = sections.join("\n\n");
                    bot.send_message(msg.chat.id, reply).await?;
                }
            }
            Err(err) => {
                tracing::error!("{err}");
            }
        }
    } else if let Some(num_str) = data.strip_prefix(SHOW_PREFIX) {
        // Show expenses for one specific traveler, identified by number.
        let Ok(number) = num_str.parse::<i64>() else {
            tracing::warn!("Invalid traveler number in callback: '{num_str}'");
            dialogue.exit().await?;
            return Ok(());
        };
        match Traveler::db_select_by_number(db.clone(), msg.chat.id, number).await {
            Ok(Some(t)) => {
                let expenses = Expense::db_select_by_payer(db, t).await;
                match expenses {
                    Ok(list) if !list.is_empty() => {
                        let reply = format_expenses(&list, ctx);
                        bot.send_message(msg.chat.id, reply).await?;
                    }
                    Ok(_) => {
                        tracing::info!("Traveler #{number} has no expenses (already deleted?)");
                    }
                    Err(err) => {
                        tracing::error!("{err}");
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("Traveler #{number} not found");
            }
            Err(err) => {
                tracing::error!("{err}");
            }
        }
    }

    dialogue.exit().await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}
