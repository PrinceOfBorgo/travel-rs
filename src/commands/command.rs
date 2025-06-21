use crate::{
    Context, HandlerResult,
    commands::{
        HelpMessage, add_traveler, delete_expense, delete_transfer, delete_traveler, help,
        list_expenses, list_transfers, list_travelers, set_currency, set_language, show_balances,
        show_expense, show_stats, transfer,
    },
    consts::MIN_SIMILARITY_SCORE,
    i18n::{self, Translate, TranslateWithArgs, help::*},
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use rust_fuzzy_search::fuzzy_search_best_n;
use std::sync::{Arc, Mutex};
use std::{str::FromStr, sync::LazyLock};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{prelude::*, utils::command::BotCommands};
use unic_langid::LanguageIdentifier;

pub static COMMANDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    Command::iter()
        .map(|variant| variant.as_ref().to_owned())
        .collect()
});

#[derive(BotCommands, Clone, EnumIter, AsRefStr, EnumString)]
#[strum(serialize_all = "lowercase")]
#[command(rename_rule = "lowercase", description = "{descr-command}")]
pub enum Command {
    #[command(description = "{descr-help}")]
    Help { command: String },
    #[command(description = "{descr-set-language}")]
    SetLanguage { langid: LanguageIdentifier },
    #[command(description = "{descr-set-currency}")]
    SetCurrency { currency: String },
    #[command(description = "{descr-add-traveler}")]
    AddTraveler { name: Name },
    #[command(description = "{descr-delete-traveler}")]
    DeleteTraveler { name: Name },
    #[command(description = "{descr-list-travelers}")]
    ListTravelers,
    #[command(description = "{descr-add-expense}")]
    AddExpense,
    #[command(description = "{descr-delete-expense}")]
    DeleteExpense { number: i64 },
    #[command(description = "{descr-list-expenses}")]
    ListExpenses { description: String },
    #[command(description = "{descr-show-expense}")]
    ShowExpense { number: i64 },
    #[command(description = "{descr-transfer}", parse_with = "split")]
    Transfer {
        from: Name,
        to: Name,
        amount: Decimal,
    },
    #[command(description = "{descr-delete-transfer}")]
    DeleteTransfer { number: i64 },
    #[command(description = "{descr-list-transfers}")]
    ListTransfers { name: Name },
    #[command(description = "{descr-show-balances}")]
    ShowBalances { name: Name },
    #[command(description = "{descr-show-stats}")]
    ShowStats,
    #[command(description = "{descr-cancel}")]
    Cancel,
}

pub enum ParseCommand {
    ValidCommandName(Command),
    BestMatch(Command),
    UnknownCommand,
}

impl Command {
    pub fn parse_cmd_name(cmd_name: &str) -> ParseCommand {
        let available_cmd_names: Vec<&str> = COMMANDS.iter().map(String::as_ref).collect();

        if available_cmd_names.contains(&cmd_name) {
            let command = Command::from_str(cmd_name)
                .unwrap_or_else(|_| panic!("Command /{cmd_name} should exist."));
            ParseCommand::ValidCommandName(command)
        } else if available_cmd_names.contains(&cmd_name.to_lowercase().as_str()) {
            let best_match = Command::from_str(cmd_name.to_lowercase().as_str())
                .unwrap_or_else(|_| panic!("Command /{} should exist.", cmd_name.to_lowercase()));
            ParseCommand::BestMatch(best_match)
        } else {
            let (best_match, best_score) =
                fuzzy_search_best_n(cmd_name, &available_cmd_names, 1)[0];

            tracing::debug!(
                "Input command: {cmd_name}, best match: {best_match}, score: {best_score}."
            );

            if best_score >= MIN_SIMILARITY_SCORE {
                let best_match = Command::from_str(best_match).unwrap_or_else(|_| {
                    panic!("Command /{} should exist.", cmd_name.to_lowercase())
                });
                ParseCommand::BestMatch(best_match)
            } else {
                ParseCommand::UnknownCommand
            }
        }
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Help {
            command: String::new(),
        }
    }
}

impl HelpMessage for Command {
    fn help_message(&self, ctx: Arc<Mutex<Context>>) -> String {
        use Command::*;
        match self {
            Help { command: _ } => HELP_HELP.translate(ctx),
            SetLanguage { langid: _ } => HELP_SET_LANGUAGE.translate_with_args(
                ctx,
                &hashmap! {
                    i18n::args::AVAILABLE_LANGS.into() =>
                        i18n::available_langs()
                        .map(|lang| format!("- {lang}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into()
                },
            ),
            SetCurrency { currency: _ } => HELP_SET_CURRENCY.translate(ctx),
            AddTraveler { name: _ } => HELP_ADD_TRAVELER.translate(ctx),
            DeleteTraveler { name: _ } => HELP_DELETE_TRAVELER.translate(ctx),
            ListTravelers => HELP_LIST_TRAVELERS.translate(ctx),
            AddExpense => HELP_ADD_EXPENSE.translate(ctx),
            DeleteExpense { number: _ } => HELP_DELETE_EXPENSE.translate(ctx),
            ListExpenses { description: _ } => HELP_LIST_EXPENSES.translate(ctx),
            ShowExpense { number: _ } => HELP_SHOW_EXPENSE.translate(ctx),
            Transfer {
                from: _,
                to: _,
                amount: _,
            } => HELP_TRANSFER.translate(ctx),
            DeleteTransfer { number: _ } => HELP_DELETE_TRANSFER.translate(ctx),
            ListTransfers { name: _ } => HELP_LIST_TRANSFERS.translate(ctx),
            ShowBalances { name: _ } => HELP_SHOW_BALANCES.translate(ctx),
            ShowStats => HELP_SHOW_STATS.translate(ctx),
            Cancel => HELP_CANCEL.translate(ctx),
        }
    }
}

pub async fn commands_handler(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    msg: Message,
    cmd: Command,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let reply = command_reply(db, &msg, &cmd, ctx).await;
    bot.send_message(msg.chat.id, reply).await?;
    Ok(())
}

pub async fn command_reply(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    cmd: &Command,
    ctx: Arc<Mutex<Context>>,
) -> String {
    use Command::*;

    let result = match cmd.clone() {
        Help { command } => help(msg, &command, ctx.clone()),
        SetLanguage { langid } => set_language(db, msg, langid, ctx.clone()).await,
        SetCurrency { currency } => set_currency(db, msg, &currency, ctx.clone()).await,
        AddTraveler { name } => add_traveler(db, msg, name, ctx.clone()).await,
        DeleteTraveler { name } => delete_traveler(db, msg, name, ctx.clone()).await,
        ListTravelers => list_travelers(db, msg, ctx.clone()).await,
        DeleteExpense { number } => delete_expense(db, msg, number, ctx.clone()).await,
        ListExpenses { description } => list_expenses(db, msg, &description, ctx.clone()).await,
        ShowExpense { number } => show_expense(db, msg, number, ctx.clone()).await,
        Transfer { from, to, amount } => transfer(db, msg, from, to, amount, ctx.clone()).await,
        DeleteTransfer { number } => delete_transfer(db, msg, number, ctx.clone()).await,
        ListTransfers { name } => list_transfers(db, msg, name, ctx.clone()).await,
        ShowBalances { name } => show_balances(db, msg, name, ctx.clone()).await,
        ShowStats => show_stats(db, msg, ctx.clone()).await,
        Cancel | AddExpense => {
            unreachable!("This command is handled before calling this function.")
        }
    };

    result.unwrap_or_else(|err| {
        format!(
            "{error_message}\n\n{help_message}",
            error_message = err.translate(ctx.clone()),
            help_message = cmd.help_message(ctx)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_cmd_name {
        use super::*;

        test! { valid_command_name,
            let cmd_name = "setlanguage";
            let parsed_cmd = Command::parse_cmd_name(cmd_name);
            assert!(matches!(
                parsed_cmd,
                ParseCommand::ValidCommandName(Command::SetLanguage { .. })
            ));
        }

        test! { best_match_wrong_case,
            let cmd_name = "SetLanguage";
            let parsed_cmd = Command::parse_cmd_name(cmd_name);
            assert!(matches!(
                parsed_cmd,
                ParseCommand::BestMatch(Command::SetLanguage { .. })
            ));
        }

        test! { best_match,
            let cmd_name = "setlang";
            let parsed_cmd = Command::parse_cmd_name(cmd_name);
            assert!(matches!(
                parsed_cmd,
                ParseCommand::BestMatch(Command::SetLanguage { .. })
            ));
        }

        test! { unknown_command,
            let cmd_name = "unknowncommand";
            let parsed_cmd = Command::parse_cmd_name(cmd_name);
            assert!(matches!(parsed_cmd, ParseCommand::UnknownCommand));
        }
    }
}
