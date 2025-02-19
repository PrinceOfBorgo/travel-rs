use crate::{utils::trace_skip_all, HandlerResult};
use macro_rules_attribute::apply;
use std::sync::Arc;
use teloxide::{dispatching::dialogue::Storage, prelude::*};
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn cancel<S, D>(bot: Bot, storage: Arc<S>, msg: Message) -> HandlerResult
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    let chat_id = msg.chat.id;
    if Arc::clone(&storage).get_dialogue(chat_id).await?.is_some() {
        Dialogue::new(storage, chat_id).exit().await?;
        bot.send_message(chat_id, "The process was cancelled.")
            .await?;
    } else {
        bot.send_message(chat_id, "There is no process to cancel.")
            .await?;
    }
    Ok(())
}
