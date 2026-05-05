//! `/showexpense` dialogue: asks the user for the expense number when the
//! command is invoked without an inline argument.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate},
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{Bot, requests::Requester, types::Message};
use tracing::Level;

#[derive(Debug, Clone)]
pub enum ShowExpenseState {
    AskNumber,
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
        i18n::dialogues::SHOW_EXPENSE_ASK_NUMBER.translate(ctx),
    )
    .await?;
    dialogue
        .update(PendingCommandState::ShowExpense(
            ShowExpenseState::AskNumber,
        ))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

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
            tracing::warn!("Invalid expense number: {text:?}");
            bot.send_message(
                msg.chat.id,
                i18n::dialogues::SHOW_EXPENSE_INVALID_NUMBER.translate(ctx),
            )
            .await?;
            return Ok(());
        }
    };

    let cmd = Command::ShowExpense {
        number: CommandArg::Provided(number),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx.clone()).await;
    bot.send_message(msg.chat.id, outcome.message()).await?;
    if outcome.is_success() {
        dialogue.exit().await?;
    } else {
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::SHOW_EXPENSE_ASK_NUMBER.translate(ctx),
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
        tests::{TestBot, helpers::cancel_ok_for},
    };

    test! { ask_number_on_empty_invocation,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showexpense");
        let response = i18n::dialogues::SHOW_EXPENSE_ASK_NUMBER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_number_invalid_reprompts,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showexpense");
        bot.dispatch().await;

        bot.update("not a number");
        let response = i18n::dialogues::SHOW_EXPENSE_INVALID_NUMBER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { receive_number_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showexpense");
        bot.dispatch().await;

        // After a not-found reply, the dialogue re-prompts so the user can retry.
        bot.update("999");
        let response = i18n::dialogues::SHOW_EXPENSE_ASK_NUMBER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { dialogue_stays_alive_after_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showexpense");
        bot.dispatch().await;

        // Not-found reply does NOT exit the dialogue.
        bot.update("999");
        bot.dispatch().await;

        // /cancel acknowledges the still-running dialogue.
        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_SHOW_EXPENSE);
        bot.test_last_message(&response).await;
    }

    test! { cancel_during_dialogue,
        let db = db().await;

        let mut bot = TestBot::new(db, "/showexpense");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_SHOW_EXPENSE);
        bot.test_last_message(&response).await;
    }
}
