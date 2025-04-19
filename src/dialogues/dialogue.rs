use crate::{
    Context, HandlerResult,
    i18n::{self, translate},
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use teloxide::{dispatching::dialogue::Storage, prelude::*};
use tracing::Level;

pub struct Dialogue<S, D>
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    _phantom1: std::marker::PhantomData<S>,
    _phantom2: std::marker::PhantomData<D>,
}

impl<S, D> Dialogue<S, D>
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    #[apply(trace_skip_all)]
    pub async fn handle_already_running(
        storage: Arc<S>,
        bot: Bot,
        msg: Message,
        ctx: Arc<Mutex<Context>>,
    ) -> HandlerResult {
        let chat_id = msg.chat.id;
        if Self::already_running(storage, msg).await? {
            bot.send_message(
                chat_id,
                translate(ctx, i18n::commands::PROCESS_ALREADY_RUNNING),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn already_running(
        storage: Arc<S>,
        msg: Message,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Arc::clone(&storage)
            .get_dialogue(msg.chat.id)
            .await?
            .is_some())
    }

    pub async fn is_already_running(storage: Arc<S>, msg: Message) -> bool {
        Self::already_running(storage, msg).await.unwrap_or(false)
    }
}
