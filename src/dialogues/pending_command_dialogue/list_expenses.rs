//! `/listexpenses` filter dialogue: asks the user for a description to filter
//! expenses by when the "Filter…" inline keyboard button is tapped.

use crate::{
    Context, HandlerResult,
    commands::{Command, command_reply},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::{
        pending_command_dialogue::{
            PendingCommandDialogue, PendingCommandState, PendingCommandStorage,
        },
        storage::DialogueRegistry,
    },
    i18n::{self, Translate, TranslateWithArgs},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    dispatching::dialogue::Dialogue,
    requests::Requester,
    types::{CallbackQuery, Message},
};
use tracing::Level;

#[derive(Debug, Clone)]
pub enum ListExpensesState {
    AskDescription,
}

/// Callback handler for the "Filter…" button on `/listexpenses`. Starts the
/// pending-command dialogue and prompts the user for a description.
pub async fn receive_filter_callback(
    _db: Arc<Surreal<Any>>,
    bot: Bot,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
    registry: DialogueRegistry,
    storage: Arc<PendingCommandStorage>,
) -> HandlerResult {
    let _ = bot.answer_callback_query(q.id.clone()).await;

    let Some(msg) = q.regular_message().cloned() else {
        return Ok(());
    };

    // Remove the keyboard from the original message.
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    // If another dialogue is already running, refuse to start a new one.
    if registry.any_running(msg.chat.id).await {
        let process_name = match registry.running_label(msg.chat.id).await {
            Some(label) => label.translate(Arc::clone(&ctx)),
            None => i18n::commands::RUNNING_PROCESS_UNKNOWN.translate(Arc::clone(&ctx)),
        };
        bot.send_message(
            msg.chat.id,
            i18n::commands::PROCESS_ALREADY_RUNNING.translate_with_args(
                ctx,
                &hashmap! { i18n::args::PROCESS.into() => process_name.into() },
            ),
        )
        .await?;
        return Ok(());
    }

    // Build the dialogue handle manually (we intentionally skip
    // `enter_dialogue` in the handler tree to avoid writing a default
    // `Start` state that would trip the `any_running` check above).
    let dialogue: PendingCommandDialogue = Dialogue::new(storage, msg.chat.id);
    dialogue
        .update(PendingCommandState::ListExpenses(
            ListExpensesState::AskDescription,
        ))
        .await?;

    bot.send_message(
        msg.chat.id,
        i18n::dialogues::LIST_EXPENSES_ASK_DESCRIPTION.translate(ctx),
    )
    .await?;

    Ok(())
}

#[apply(trace_state_db)]
pub async fn receive_description(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    if text.is_empty() {
        bot.send_message(
            msg.chat.id,
            i18n::dialogues::LIST_EXPENSES_ASK_DESCRIPTION.translate(ctx),
        )
        .await?;
        return Ok(());
    }

    let cmd = Command::ListExpenses {
        description: text.to_owned(),
    };
    let outcome = command_reply(db, &msg, &cmd, ctx).await;
    bot.send_message(msg.chat.id, outcome.into_message())
        .await?;
    dialogue.exit().await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        dialogues::pending_command_dialogue::PendingCommandState,
        i18n::{self, Translate},
        tests::{TestBot, helpers},
    };
    use teloxide::dispatching::dialogue::Storage;

    /// Seeds the `ListExpenses(AskDescription)` dialogue state into the
    /// pending-command storage, simulating what the "Filter…" callback
    /// handler does. After this call the message-driven handler branch
    /// will pick up follow-up messages.
    async fn seed_list_expenses_dialogue(bot: &TestBot) {
        let storage = bot.pending_command_storage();
        storage
            .update_dialogue(
                bot.chat_id(),
                PendingCommandState::ListExpenses(super::ListExpensesState::AskDescription),
            )
            .await
            .unwrap();
    }

    test! { receive_description_filters_expenses,
        let db = db().await;
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        helpers::add_expense(&mut bot, "Toll road", rust_decimal::Decimal::new(10, 0), "Alice", &["all"]).await;
        helpers::add_expense(&mut bot, "Dinner", rust_decimal::Decimal::new(50, 0), "Alice", &["all"]).await;

        // Seed the dialogue state as if the "Filter…" button was tapped.
        seed_list_expenses_dialogue(&bot).await;

        // Feed a description filter.
        bot.update("Toll");
        let msg = bot.dispatch_and_last_message().await;
        let text = msg.unwrap();
        // The filtered output should mention "Toll road" but not "Dinner".
        assert!(text.contains("Toll road"), "Expected 'Toll road' in: {text}");
        assert!(!text.contains("Dinner"), "Did not expect 'Dinner' in: {text}");
    }

    test! { receive_description_empty_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        seed_list_expenses_dialogue(&bot).await;

        // Empty input should re-prompt.
        bot.update("   ");
        let response = i18n::dialogues::LIST_EXPENSES_ASK_DESCRIPTION.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { dialogue_exits_after_completion,
        let db = db().await;
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        seed_list_expenses_dialogue(&bot).await;

        // Provide a description (no matching expenses is fine — the dialogue
        // should exit regardless).
        bot.update("Nonexistent");
        bot.dispatch().await;

        // After completion `/cancel` reports nothing to cancel.
        bot.update("/cancel");
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { cancel_during_dialogue,
        let db = db().await;
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        seed_list_expenses_dialogue(&bot).await;

        bot.update("/cancel");
        let response = crate::tests::helpers::cancel_ok_for(
            i18n::commands::RUNNING_PROCESS_LIST_EXPENSES,
        );
        bot.test_last_message(&response).await;
    }
}
