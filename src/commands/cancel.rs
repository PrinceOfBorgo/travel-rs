use crate::{HandlerResult, i18n::translate, utils::trace_skip_all};
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
        bot.send_message(chat_id, translate(chat_id, "i18n-cancel-ok").await)
            .await?;
    } else {
        bot.send_message(
            chat_id,
            translate(chat_id, "i18n-cancel-no-process-to-cancel").await,
        )
        .await?;
    }
    Ok(())
}
