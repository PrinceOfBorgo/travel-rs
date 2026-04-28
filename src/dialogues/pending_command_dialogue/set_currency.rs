//! `/setcurrency` dialogue: asks the user for the currency code when the
//! command is invoked without an inline argument.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate},
    trace_state, trace_state_db,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{Bot, requests::Requester, types::Message};
use tracing::Level;

#[derive(Debug, Clone)]
pub enum SetCurrencyState {
    AskCurrency,
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
        i18n::dialogues::SET_CURRENCY_ASK_CURRENCY.translate(ctx),
    )
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
            i18n::dialogues::SET_CURRENCY_ASK_CURRENCY.translate(ctx),
        )
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
        tests::TestBot,
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
        let response = i18n::commands::CANCEL_OK.translate_default();
        bot.test_last_message(&response).await;
    }
}
