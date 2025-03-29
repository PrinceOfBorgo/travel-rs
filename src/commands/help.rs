use crate::{
    Context,
    commands::{Command, HelpMessage, ParseCommand},
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{commands::COMMAND_DESCRIPTIONS, translate},
    trace_command,
};
use macro_rules_attribute::apply;
use std::sync::{Arc, Mutex};
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

    let cmd_name = command.strip_prefix('/').unwrap_or(command).to_lowercase();

    match Command::parse_cmd_name(&cmd_name) {
        ParseCommand::ValidCommandName(command) => {
            tracing::debug!(DEBUG_SUCCESS);
            Ok(command.help_message(ctx).to_string())
        }
        ParseCommand::BestMatch(best_match) => {
            let err = CommandError::Help {
                command: command.to_owned(),
                best_match: Some(best_match.as_ref().to_string()),
            };
            tracing::error!("{err}");
            Err(err)
        }
        ParseCommand::UnknownCommand => {
            let err = CommandError::Help {
                command: command.to_owned(),
                best_match: None,
            };
            tracing::error!("{err}");
            Err(err)
        }
    }
}
