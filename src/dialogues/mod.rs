use crate::{
    Context, HandlerResult,
    i18n::{self, translate},
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use teloxide::{dispatching::dialogue::Storage, prelude::*};
use tracing::Level;

pub mod add_expense_dialogue;

#[apply(trace_skip_all)]
pub async fn handle_process_already_running<S, D>(
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
        bot.send_message(
            chat_id,
            translate(ctx, i18n::commands::PROCESS_ALREADY_RUNNING),
        )
        .await?;
    }
    Ok(())
}

pub async fn process_already_running<S, D>(storage: Arc<S>, msg: Message) -> bool
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    Arc::clone(&storage)
        .get_dialogue(msg.chat.id)
        .await
        .is_ok_and(|dialogue| dialogue.is_some())
}
