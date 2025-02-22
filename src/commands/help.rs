use crate::{
    commands::{Command, HelpMessage},
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    trace_command,
};
use macro_rules_attribute::apply;
use strum::IntoEnumIterator;
use teloxide::{prelude::*, utils::command::BotCommands};
use tracing::Level;

#[apply(trace_command)]
pub fn help(msg: &Message, command: &str) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let command = command.trim();
    if command.is_empty() {
        tracing::debug!(DEBUG_SUCCESS);
        return Ok(Command::descriptions().to_string());
    }

    match Command::iter()
        .find(|variant| variant.as_ref() == command.trim_matches('/').to_lowercase())
        .map(|variant| variant.help_message())
    {
        Some(help) => {
            tracing::debug!(DEBUG_SUCCESS);
            Ok(help.to_string())
        }
        None => {
            tracing::error!("No help available for command /{command}.");
            Err(CommandError::Help {
                command: command.to_owned(),
            })
        }
    }
}
