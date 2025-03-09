use crate::{
    Context, HandlerResult,
    commands::{COMMANDS, Command, HelpMessage},
    consts::{BOT_NAME, MIN_SIMILARITY_SCORE},
    i18n,
    i18n::translate_with_args,
    utils::trace_skip_all,
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use rust_fuzzy_search::fuzzy_search_best_n;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_skip_all)]
pub async fn unknown_command(bot: Bot, msg: Message, ctx: Arc<Mutex<Context>>) -> HandlerResult {
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
            .help_message(ctx.clone());

        bot.send_message(
            msg.chat.id,
            translate_with_args(
                ctx,
                i18n::commands::INVALID_COMMAND_USAGE,
                &hashmap! {
                i18n::args::COMMAND.into() => command.into(),
                i18n::args::HELP_MESSAGE.into() => help_message.into()},
            ),
        )
        .await?;
    } else if available_cmd_names.contains(&cmd_name.to_lowercase().as_str()) {
        bot.send_message(
            msg.chat.id,
            translate_with_args(
                ctx,
                i18n::commands::UNKNOWN_COMMAND_BEST_MATCH,
                &hashmap! {
                i18n::args::COMMAND.into() => text.into(),
                i18n::args::BEST_MATCH.into() => cmd_name.to_lowercase().into()},
            ),
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
                    ctx,
                    i18n::commands::UNKNOWN_COMMAND_BEST_MATCH,
                    &hashmap! {
                        i18n::args::COMMAND.into() => text.into(),
                        i18n::args::BEST_MATCH.into() => best_match.into()
                    },
                ),
            )
            .await?;
        } else {
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
