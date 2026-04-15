use crate::{
    Context,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, TranslateWithArgs},
    settings::SETTINGS,
    trace_command,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub fn app(msg: &Message, ctx: Arc<Mutex<Context>>) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");

    // Check if miniapp_url is configured
    let miniapp_url = SETTINGS
        .api
        .as_ref()
        .and_then(|api| api.miniapp_url.as_ref())
        .ok_or(CommandError::MiniAppNotConfigured)?;

    // Generate a start parameter with the chat_id for context
    let start_param = format!("chat_{}", msg.chat.id);
    let full_url = format!("{miniapp_url}?startapp={start_param}");

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(i18n::commands::APP_OK
        .translate_with_args(ctx, &hashmap! {i18n::args::URL.into() => full_url.into()}))
}

#[cfg(test)]
mod tests {
    // Tests would require mocking SETTINGS which is not straightforward
    // The command is simple enough that integration tests suffice
}
