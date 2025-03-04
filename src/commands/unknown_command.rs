use crate::{
    HandlerResult,
    commands::{COMMANDS, Command, HelpMessage},
    consts::{BOT_NAME, MIN_SIMILARITY_SCORE},
    i18n::translate_with_args,
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use rust_fuzzy_search::fuzzy_search_best_n;
use std::str::FromStr;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn unknown_command(bot: Bot, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let command = text.split_whitespace().next().unwrap_or("");

    let Some(mut cmd_name) = text.strip_prefix('/') else {
        return Ok(());
    };

    cmd_name = cmd_name
        .strip_suffix(&format!("@{BOT_NAME}"))
        .unwrap_or(cmd_name);

    let available_cmd_names: Vec<&str> = COMMANDS.iter().map(String::as_ref).collect();

    if available_cmd_names.contains(&cmd_name) {
        let help_message = Command::from_str(cmd_name)
            .unwrap_or_else(|_| panic!("Command {command} should exist."))
            .help_message();

        bot.send_message(
            msg.chat.id,
            translate_with_args(
                msg.chat.id,
                "i18n-invalid-command-usage",
                &hashmap![
                    "command".into() => command.into(), 
                    "help_message".into() => help_message.into()],
            )
            .await,
        )
        .await?;
    } else if available_cmd_names.contains(&cmd_name.to_lowercase().as_str()) {
        bot.send_message(
            msg.chat.id,
            translate_with_args(
                msg.chat.id,
                "i18n-unknown-command-best-match",
                &hashmap![
                    "command".into() => text.into(), 
                    "best-match".into() => cmd_name.to_lowercase().into()],
            )
            .await,
        )
        .await?;
    } else {
        let (best_match, best_score) = fuzzy_search_best_n(cmd_name, &available_cmd_names, 1)[0];

        tracing::debug!(
            "Input command: {cmd_name}, best match: {best_match}, score: {best_score}."
        );

        if best_score >= MIN_SIMILARITY_SCORE {
            bot.send_message(
                msg.chat.id,
                translate_with_args(
                    msg.chat.id,
                    "i18n-unknown-command-best-match",
                    &hashmap![
                    "command".into() => text.into(),
                    "best-match".into() => best_match.into()
                    ],
                )
                .await,
            )
            .await?;
        } else {
            bot.send_message(
                msg.chat.id,
                translate_with_args(
                    msg.chat.id,
                    "i18n-unknown-command",
                    &hashmap!["command".into() => text.into()],
                )
                .await,
            )
            .await?;
        }
    }

    Ok(())
}
