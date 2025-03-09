use crate::{
    Context,
    commands::{Command, HelpMessage},
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{commands::COMMAND_DESCRIPTIONS, translate},
    trace_command,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub fn help(
    msg: &Message,
    command: &str,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let command = command.trim();
    if command.is_empty() {
        tracing::debug!(DEBUG_SUCCESS);
        return Ok(translate(ctx, COMMAND_DESCRIPTIONS));
    }

    match Command::iter()
        .find(|variant| variant.as_ref() == command.trim_matches('/').to_lowercase())
        .map(|variant| variant.help_message(ctx))
    {
        Some(help) => {
            tracing::debug!(DEBUG_SUCCESS);
            Ok(help.to_string())
        }
        None => {
            let err = CommandError::Help {
                command: command.to_owned(),
            };
            tracing::error!("{err}");
            Err(err)
        }
    }
}
