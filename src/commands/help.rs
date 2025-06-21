use crate::{
    Context,
    commands::{Command, HelpMessage, ParseCommand},
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{Translate, commands::COMMAND_DESCRIPTIONS},
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
    tracing::debug!("{LOG_DEBUG_START}");
    let command = command.trim();
    if command.is_empty() {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(COMMAND_DESCRIPTIONS.translate(ctx));
    }

    let cmd_name = command.strip_prefix('/').unwrap_or(command).to_lowercase();

    match Command::parse_cmd_name(&cmd_name) {
        ParseCommand::ValidCommandName(command) => {
            tracing::debug!("{LOG_DEBUG_SUCCESS}");
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

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        errors::CommandError,
        i18n::{self, Translate},
        tests::TestBot,
    };

    test! { help_ok,
        let db = db().await;

        let mut bot = TestBot::new(db, "/help");
        let response = i18n::commands::COMMAND_DESCRIPTIONS.translate_default();
        bot.test_last_message(&response).await;

        bot.update("/help addtraveler");
        let response = i18n::help::HELP_ADD_TRAVELER.translate_default();
        bot.test_last_message(&response).await;

        bot.update("/help /addtraveler");
        let response = i18n::help::HELP_ADD_TRAVELER.translate_default();
        bot.test_last_message(&response).await;

        bot.update("/help   addtraveler  ");
        let response = i18n::help::HELP_ADD_TRAVELER.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { help_best_match,
        let db = db().await;

        let mut bot = TestBot::new(db, "/help addtrave");
        let err = CommandError::Help {
            command: String::from("addtrave"),
            best_match: Some(String::from("addtraveler")),
        }
        .translate_default();
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );
    }

    test! { help_unknown_command,
        let db = db().await;

        let mut bot = TestBot::new(db, "/help unknowncommand");
        let err = CommandError::Help {
            command: String::from("unknowncommand"),
            best_match: None,
        }
        .translate_default();
        assert!(
            bot.dispatch_and_last_message()
                .await
                .is_some_and(|msg| msg.starts_with(&err))
        );
    }
}
