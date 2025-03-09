use crate::{
    Context, HandlerResult,
    commands::{set_language, show_balance, show_balances, show_expense},
    i18n::{self, Translatable, help::*, translate, translate_with_args},
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use teloxide::{prelude::*, utils::command::BotCommands};
use unic_langid::LanguageIdentifier;

use crate::commands::{
    HelpMessage, add_traveler, delete_expense, delete_traveler, find_expenses, help, list_expenses,
    list_travelers, transfer,
};

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
    ListExpenses,
    #[command(description = "{descr-find-expenses}")]
    FindExpenses { description: String },
    #[command(description = "{descr-show-expense}")]
    ShowExpense { number: i64 },
    #[command(description = "{descr-transfer}", parse_with = "split")]
    Transfer {
        from: Name,
        to: Name,
        amount: Decimal,
    },
    #[command(description = "{descr-show-balance}")]
    ShowBalance { name: Name },
    #[command(description = "{descr-show-balances}")]
    ShowBalances,
    #[command(description = "{descr-cancel}")]
    Cancel,
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
            Help { command: _ } => translate(ctx, HELP_HELP),
            SetLanguage { langid: _ } => translate_with_args(
                ctx,
                HELP_SET_LANGUAGE,
                &hashmap! {
                    i18n::args::AVAILABLE_LANGS.into() =>
                    i18n::available_langs()
                    .map(|lang| format!("- {lang}"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .into()
                },
            ),
            AddTraveler { name: _ } => translate(ctx, HELP_ADD_TRAVELER),
            DeleteTraveler { name: _ } => translate(ctx, HELP_DELETE_TRAVELER),
            ListTravelers => translate(ctx, HELP_LIST_TRAVELERS),
            AddExpense => translate(ctx, HELP_ADD_EXPENSE),
            DeleteExpense { number: _ } => translate(ctx, HELP_DELETE_EXPENSE),
            ListExpenses => translate(ctx, HELP_LIST_EXPENSES),
            FindExpenses { description: _ } => translate(ctx, HELP_FIND_EXPENSES),
            ShowExpense { number: _ } => translate(ctx, HELP_SHOW_EXPENSE),
            Transfer {
                from: _,
                to: _,
                amount: _,
            } => translate(ctx, HELP_TRANSFER),
            ShowBalance { name: _ } => translate(ctx, HELP_SHOW_BALANCE),
            ShowBalances => translate(ctx, HELP_SHOW_BALANCES),
            Cancel => translate(ctx, HELP_CANCEL),
        }
    }
}

pub async fn commands_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    use Command::*;

    let result = match cmd.clone() {
        Help { command } => help(&msg, &command, ctx.clone()),
        SetLanguage { langid } => set_language(&msg, langid, ctx.clone()).await,
        AddTraveler { name } => add_traveler(&msg, name, ctx.clone()).await,
        DeleteTraveler { name } => delete_traveler(&msg, name, ctx.clone()).await,
        ListTravelers => list_travelers(&msg, ctx.clone()).await,
        DeleteExpense { number } => delete_expense(&msg, number, ctx.clone()).await,
        ListExpenses => list_expenses(&msg, ctx.clone()).await,
        FindExpenses { description } => find_expenses(&msg, &description, ctx.clone()).await,
        ShowExpense { number } => show_expense(&msg, number, ctx.clone()).await,
        Transfer { from, to, amount } => transfer(&msg, from, to, amount, ctx.clone()).await,
        ShowBalance { name } => show_balance(&msg, name, ctx.clone()).await,
        ShowBalances => show_balances(&msg, ctx.clone()).await,
        Cancel | AddExpense => {
            unreachable!("This command is handled before calling this function.")
        }
    };

    match result {
        Ok(reply) => {
            bot.send_message(msg.chat.id, reply).await?;
        }
        Err(err) => {
            let reply = format!(
                "{error_message}\n\n{help_message}",
                error_message = err.translate(ctx.clone()),
                help_message = cmd.help_message(ctx)
            );
            bot.send_message(msg.chat.id, reply).await?;
        }
    }

    Ok(())
}
