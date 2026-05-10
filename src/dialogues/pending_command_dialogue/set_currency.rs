//! `/setcurrency` dialogue: asks the user for the currency code when the
//! command is invoked without an inline argument.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate},
    keyboard::{self, DEFAULT_ROWS_PER_PAGE, PaginatedKeyboardConfig, PickerItem},
    money_wrapper::currency_label,
    settings::SETTINGS,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardMarkup, Message},
};
use tracing::Level;

/// Prefix used to identify callback queries originating from the
/// `/setcurrency` inline keyboard. The remainder of the callback data is
/// the selected currency code (e.g. `setcur:USD`).
pub const CALLBACK_PREFIX: &str = "setcur:";

/// Callback data used by the cancel button in the `/setcurrency` inline
/// keyboard.
pub const CANCEL_CALLBACK_DATA: &str = "setcur:__cancel__";

/// Callback data for blank spacer buttons in the `/setcurrency` keyboard.
const NOOP_CALLBACK_DATA: &str = "setcur:__noop__";

/// Number of currency buttons per row in the inline keyboard.
const CURRENCIES_PER_ROW: usize = 2;

#[derive(Debug, Clone)]
pub enum SetCurrencyState {
    AskCurrency,
}

/// Builds an inline keyboard with the popular currencies in a uniform grid.
fn popular_currencies_keyboard(ctx: Arc<Mutex<Context>>) -> InlineKeyboardMarkup {
    let items: Vec<PickerItem> = SETTINGS
        .i18n
        .popular_currencies
        .iter()
        .map(|code| PickerItem {
            label: currency_label(code),
            value: code.to_string(),
        })
        .collect();

    // Popular-currencies list is small — always fits on one page.
    keyboard::paginated_keyboard(PaginatedKeyboardConfig {
        items: &items,
        page: 0,
        columns: CURRENCIES_PER_ROW,
        rows_per_page: DEFAULT_ROWS_PER_PAGE,
        prefix: CALLBACK_PREFIX,
        cancel_callback: CANCEL_CALLBACK_DATA,
        noop_callback: NOOP_CALLBACK_DATA,
        action_buttons: &[],
        show_cancel: true,
        ctx,
    })
    .expect("at least one popular currency must be configured")
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
    tracing::info!("Dialogue started: /setcurrency");
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
            cancel_callback: CANCEL_CALLBACK_DATA,
            noop_callback: NOOP_CALLBACK_DATA,
            prefix: CALLBACK_PREFIX,
            running_process_key: i18n::commands::RUNNING_PROCESS_SET_CURRENCY,
        },
    )
    .await?;

    let keyboard::CallbackAction::Selection {
        value: currency,
        msg,
    } = action
    else {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    };

    let cmd = Command::SetCurrency {
        currency: CommandArg::Provided(currency),
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
