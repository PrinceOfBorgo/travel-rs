//! `/setcurrency` dialogue: asks the user for the currency code when the
//! command is invoked without an inline argument.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TranslateWithArgs, TryTranslate},
    trace_state, trace_state_db,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use tracing::Level;

/// Prefix used to identify callback queries originating from the
/// `/setcurrency` inline keyboard. The remainder of the callback data is
/// the selected currency code (e.g. `setcur:USD`).
pub const CALLBACK_PREFIX: &str = "setcur:";

/// Callback data used by the cancel button in the `/setcurrency` inline
/// keyboard.
pub const CANCEL_CALLBACK_DATA: &str = "setcur:__cancel__";

/// Curated short list of widely-used currencies surfaced as quick-pick
/// buttons. Users can still type any ISO 4217 or supported crypto code as a
/// free-text fallback.
const POPULAR_CURRENCIES: &[&str] = &["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "CNY"];

/// Number of currency buttons per row in the inline keyboard.
const CURRENCIES_PER_ROW: usize = 2;

#[derive(Debug, Clone)]
pub enum SetCurrencyState {
    AskCurrency,
}

/// Builds an inline keyboard with the popular currencies laid out in a
/// compact grid plus a localized cancel button on its own row.
fn popular_currencies_keyboard(ctx: Arc<Mutex<Context>>) -> InlineKeyboardMarkup {
    let buttons: Vec<InlineKeyboardButton> = POPULAR_CURRENCIES
        .iter()
        .map(|code| {
            // Fall back to the bare code when no localized label is defined.
            let label = format!("{}{code}", i18n::labels::CURRENCY_LABEL_PREFIX)
                .try_translate(ctx.clone())
                .unwrap_or_else(|| (*code).to_owned());
            InlineKeyboardButton::callback(label, format!("{CALLBACK_PREFIX}{code}"))
        })
        .collect();

    let mut rows: Vec<Vec<InlineKeyboardButton>> = buttons
        .chunks(CURRENCIES_PER_ROW)
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
        i18n::dialogues::SET_CURRENCY_ASK_CURRENCY.translate(ctx.clone()),
    )
    .reply_markup(popular_currencies_keyboard(ctx))
    .await?;
    dialogue
        .update(PendingCommandState::SetCurrency(
            SetCurrencyState::AskCurrency,
        ))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[apply(trace_state_db)]
pub async fn receive_currency(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    if text.is_empty() {
        tracing::warn!("Empty currency input.");
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::SET_CURRENCY_INVALID_CURRENCY.translate(ctx),
        )
        .await?;
        return Ok(());
    }

    let cmd = Command::SetCurrency {
        currency: CommandArg::Provided(text.to_owned()),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx.clone()).await;
    bot.send_message(msg.chat.id, outcome.message()).await?;
    if outcome.is_success() {
        dialogue.exit().await?;
    } else {
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::SET_CURRENCY_ASK_CURRENCY.translate(ctx.clone()),
        )
        .reply_markup(popular_currencies_keyboard(ctx))
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

    // Parse the callback data: it's either the cancel sentinel or a currency
    // code prefixed with `CALLBACK_PREFIX`.
    let data = q.data.as_deref().unwrap_or("");

    if data == CANCEL_CALLBACK_DATA {
        // Remove the inline keyboard from the original prompt (best-effort).
        let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;
        let process_name =
            i18n::commands::RUNNING_PROCESS_SET_CURRENCY.translate(Arc::clone(&ctx));
        let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
            ctx,
            &maplit::hashmap! { i18n::args::PROCESS.into() => process_name.into() },
        );
        bot.send_message(msg.chat.id, cancel_msg).await?;
        dialogue.exit().await?;
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    }

    let currency = data.strip_prefix(CALLBACK_PREFIX).unwrap_or("");
    if currency.is_empty() {
        tracing::warn!("Empty currency in callback data");
        return Ok(());
    }

    let cmd = Command::SetCurrency {
        currency: CommandArg::Provided(currency.to_owned()),
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
            i18n::dialogues::SET_CURRENCY_ASK_CURRENCY.translate(ctx.clone()),
        )
        .reply_markup(popular_currencies_keyboard(ctx))
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

    test! { ask_currency_on_empty_invocation,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setcurrency");
        let response = i18n::dialogues::SET_CURRENCY_ASK_CURRENCY.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_currency_empty_reprompts,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setcurrency");
        bot.dispatch().await;

        bot.update("   ");
        let response = i18n::dialogues::SET_CURRENCY_INVALID_CURRENCY.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { cancel_during_dialogue,
        let db = db().await;

        let mut bot = TestBot::new(db, "/setcurrency");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_SET_CURRENCY);
        bot.test_last_message(&response).await;
    }
}
