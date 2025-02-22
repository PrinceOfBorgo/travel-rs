use crate::{
    commands::{Command, HelpMessage, COMMANDS},
    consts::{BOT_NAME, MIN_SIMILARITY_SCORE},
    utils::trace_skip_all,
    HandlerResult,
};
use macro_rules_attribute::apply;
use rust_fuzzy_search::fuzzy_search_best_n;
use std::str::FromStr;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn unknown_command(bot: Bot, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let Some(mut command) = text.strip_prefix('/') else {
        return Ok(());
    };

    command = command.split_whitespace().next().unwrap_or("");

    let Some(command) = command.strip_suffix(&format!("@{BOT_NAME}")) else {
        return Ok(());
    };

    let available_commands: Vec<&str> = COMMANDS.iter().map(String::as_ref).collect();

    if available_commands.contains(&command) {
        let help_message = Command::from_str(command)
            .unwrap_or_else(|_| panic!("Command /{} should exist.", command))
            .help_message();

        bot.send_message(
            msg.chat.id,
            format!(
                "Invalid usage of command: /{}.\n\n{}",
                command, help_message
            ),
        )
        .await?;
    } else if available_commands.contains(&command.to_lowercase().as_str()) {
        bot.send_message(
            msg.chat.id,
            format!(
                "Unknown command: {}.\nDid you mean: /{}?",
                text,
                command.to_lowercase()
            ),
        )
        .await?;
    } else {
        let (best_match, best_score) = fuzzy_search_best_n(command, &available_commands, 1)[0];

        tracing::debug!(
            "Input command: {}, best match: {}, score: {}.",
            command,
            best_match,
            best_score
        );

        if best_score >= MIN_SIMILARITY_SCORE {
            bot.send_message(
                msg.chat.id,
                format!("Unknown command: {}.\nDid you mean: /{}?", text, best_match),
            )
            .await?;
        } else {
            bot.send_message(msg.chat.id, format!("Unknown command: {}.", text))
                .await?;
        }
    }

    Ok(())
}
