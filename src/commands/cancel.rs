use crate::{
    Context, HandlerResult,
    dialogues::storage::DialogueRegistry,
    i18n::{self, Translate},
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
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
    let cancelled_any = registry.exit_all(chat_id).await?;

    let key = if cancelled_any {
        i18n::commands::CANCEL_OK
    } else {
        i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL
    };
    bot.send_message(chat_id, key.translate(ctx)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate},
        tests::TestBot,
    };

    test! { cancel_ok,
        let db = db().await;

        // Start process
        let mut bot = TestBot::new(db, "/addexpense");
        bot.dispatch().await;

        // Cancel process
        bot.update("/cancel");
        let response = i18n::commands::CANCEL_OK.translate_default();
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
        let response = i18n::commands::CANCEL_OK.translate_default();
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
        let response = i18n::commands::CANCEL_OK.translate_default();
        bot.test_last_message(&response).await;

        // After cancel the chat is back to a clean state
        bot.update("/cancel");
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
        bot.test_last_message(&response).await;
    }
}
