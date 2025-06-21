use crate::{
    Context, HandlerResult,
    i18n::{self, Translate},
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use teloxide::{dispatching::dialogue::Storage, prelude::*};
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn cancel<S, D>(
    bot: Bot,
    storage: Arc<S>,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    let chat_id = msg.chat.id;
    if Arc::clone(&storage).get_dialogue(chat_id).await?.is_some() {
        Dialogue::new(storage, chat_id).exit().await?;
        bot.send_message(chat_id, i18n::commands::CANCEL_OK.translate(ctx))
            .await?;
    } else {
        bot.send_message(
            chat_id,
            i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate(ctx),
        )
        .await?;
    }
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
}
