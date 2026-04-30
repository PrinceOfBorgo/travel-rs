use crate::{
    Context, HandlerResult,
    dialogues::storage::DialogueRegistry,
    i18n::{self, Translate, TranslateWithArgs, args},
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn cancel(
    bot: Bot,
    registry: DialogueRegistry,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    // Capture the running dialogue's label *before* exiting so the cancel
    // confirmation can name the process that was just cancelled.
    let running_label = registry.running_label(chat_id).await;
    let cancelled_any = registry.exit_all(chat_id).await?;

    let text = if cancelled_any {
        let process_name = match running_label {
            Some(label) => label.translate(Arc::clone(&ctx)),
            // Race: the dialogue exited between `running_label` and `exit_all`.
            None => i18n::commands::RUNNING_PROCESS_UNKNOWN.translate(Arc::clone(&ctx)),
        };
        i18n::commands::CANCEL_OK.translate_with_args(
            ctx,
            &hashmap! { args::PROCESS.into() => process_name.into() },
        )
    } else {
        i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate(ctx)
    };
    bot.send_message(chat_id, text).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate},
        tests::{TestBot, helpers::cancel_ok_for},
    };

    test! { cancel_ok,
        let db = db().await;

        // Start process
        let mut bot = TestBot::new(db, "/addexpense");
        bot.dispatch().await;

        // Cancel process
        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_ADD_EXPENSE);
        bot.test_last_message(&response).await;
    }

    test! { cancel_no_process_to_cancel,
        let db = db().await;

        let mut bot = TestBot::new(db, "/cancel");
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { cancel_twice,
        let db = db().await;

        // Start process
        let mut bot = TestBot::new(db, "/addexpense");
        bot.dispatch().await;

        // Cancel process -> ok
        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_ADD_EXPENSE);
        bot.test_last_message(&response).await;

        // Cancel again -> no process to cancel
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { cancel_pending_command_dialogue,
        let db = db().await;

        // Start the AddTraveler pending-command dialogue
        let mut bot = TestBot::new(db, "/addtraveler");
        bot.dispatch().await;

        // Cancel it
        bot.update("/cancel");
        let response = cancel_ok_for(i18n::commands::RUNNING_PROCESS_ADD_TRAVELER);
        bot.test_last_message(&response).await;

        // After cancel the chat is back to a clean state
        bot.update("/cancel");
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
        bot.test_last_message(&response).await;
    }
}
