use crate::{Context, HandlerResult, i18n, i18n::translate, utils::trace_skip_all};
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
        bot.send_message(chat_id, translate(ctx, i18n::commands::CANCEL_OK))
            .await?;
    } else {
        bot.send_message(
            chat_id,
            translate(ctx, i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL),
        )
        .await?;
    }
    Ok(())
}
