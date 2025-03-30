use super::ParseCommand;
use crate::{
    Context, HandlerResult,
    commands::{Command, HelpMessage},
    i18n,
    i18n::translate_with_args,
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn unknown_command(bot: Bot, msg: Message, ctx: Arc<Mutex<Context>>) -> HandlerResult {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let command = text.split_whitespace().next().unwrap_or("");
    let Some(mut cmd_name) = command.strip_prefix('/') else {
        return Ok(());
    };
    cmd_name = cmd_name
        .strip_suffix(&format!(
            "@{bot_name}",
            bot_name = bot
                .get_me()
                .await?
                .user
                .username
                .expect("Bots must have a username")
        ))
        .unwrap_or(cmd_name);

    match Command::parse_cmd_name(cmd_name) {
        ParseCommand::ValidCommandName(command) => {
            let help_message = command.help_message(ctx.clone());

            bot.send_message(
                msg.chat.id,
                translate_with_args(
                    ctx,
                    i18n::commands::INVALID_COMMAND_USAGE,
                    &hashmap! {
                        i18n::args::COMMAND.into() => format!("/{cmd_name}").into(),
                        i18n::args::HELP_MESSAGE.into() => help_message.into()
                    },
                ),
            )
            .await?;
        }
        ParseCommand::BestMatch(best_match) => {
            bot.send_message(
                msg.chat.id,
                translate_with_args(
                    ctx,
                    i18n::commands::UNKNOWN_COMMAND_BEST_MATCH,
                    &hashmap! {
                        i18n::args::COMMAND.into() => text.into(),
                        i18n::args::BEST_MATCH.into() => format!("/{}", best_match.as_ref()).into()
                    },
                ),
            )
            .await?;
        }
        ParseCommand::UnknownCommand => {
            bot.send_message(
                msg.chat.id,
                translate_with_args(
                    ctx,
                    i18n::commands::UNKNOWN_COMMAND,
                    &hashmap! {i18n::args::COMMAND.into() => text.into()},
                ),
            )
            .await?;
        }
    }

    Ok(())
}
