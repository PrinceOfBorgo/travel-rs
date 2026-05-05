//! `/deletetraveler` dialogue: asks the user for the traveler's name when
//! the command is invoked without an inline argument, then delegates to the
//! regular command handler.

use crate::{
    Context, HandlerResult,
    commands::{Command, CommandArg, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate},
    traveler::Name,
};
use macro_rules_attribute::apply;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{Bot, requests::Requester, types::Message};
use tracing::Level;

#[derive(Debug, Clone)]
pub enum DeleteTravelerState {
    AskName,
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
        i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate(ctx),
    )
    .await?;
    dialogue
        .update(PendingCommandState::DeleteTraveler(
            DeleteTravelerState::AskName,
        ))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[apply(trace_state_db)]
pub async fn receive_name(
    db: Arc<Surreal<Any>>,
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

    let cmd = Command::DeleteTraveler {
        name: CommandArg::Provided(name),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx.clone()).await;
    bot.send_message(msg.chat.id, outcome.message()).await?;
    if outcome.is_success() {
        dialogue.exit().await?;
    } else {
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate(ctx),
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

        bot.update("Alice");
        let response = i18n::commands::DELETE_TRAVELER_OK.translate_with_args_default(
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
        let response = i18n::commands::DELETE_TRAVELER_OK.translate_with_args_default(
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

        // Dialogue is still active: a follow-up "not found" target is
        // reported and the dialogue re-prompts the user.
        bot.update("Alice");
        let response = i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate_default();
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

    test! { dialogue_exits_after_completion,
        let db = db().await;

        // Add traveler "Alice" so the delete actually completes.
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        bot.update("/deletetraveler");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;

        // After completion `/cancel` reports nothing to cancel.
        bot.update("/cancel");
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
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
