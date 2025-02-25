use crate::{
    HandlerResult,
    commands::{show_balance, show_balances, show_expense},
    consts::{ALL_KWORD, SPLIT_AMONG_ENTRIES_SEP, SPLIT_AMONG_NAME_AMOUNT_SEP},
    traveler::Name,
};
use rust_decimal::Decimal;
use std::sync::LazyLock;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use teloxide::{prelude::*, utils::command::BotCommands};

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
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(
        description = "show an help message for the specified command. If no command is specified, show this text."
    )]
    Help { command: String },
    #[command(description = "add a traveler to the travel plan.")]
    AddTraveler { name: Name },
    #[command(description = "delete a traveler from the travel plan.")]
    DeleteTraveler { name: Name },
    #[command(description = "show the travelers in the travel plan.")]
    ListTravelers,
    #[command(description = "add a new expense to the travel plan.")]
    AddExpense,
    #[command(
        description = "delete the expense with the specified identifying number from the travel plan."
    )]
    DeleteExpense { number: i64 },
    #[command(description = "show the expenses in the travel plan.")]
    ListExpenses,
    #[command(description = "find the expenses matching the specified description.")]
    FindExpenses { description: String },
    #[command(
        description = "show the details of the expense with the specified identifying number."
    )]
    ShowExpense { number: i64 },
    #[command(
        description = "transfer the specified amount from one traveler to another.",
        parse_with = "split"
    )]
    Transfer {
        from: Name,
        to: Name,
        amount: Decimal,
    },
    #[command(description = "show the simplified balance of the specified traveler.")]
    ShowBalance { name: Name },
    #[command(description = "show the simplified balances of all travelers.")]
    ShowBalances,
    #[command(description = "cancel the currently running interactive command.")]
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
    fn help_message(&self) -> String {
        use Command::*;
        match self {
            Help { command: _ } => format!(
"/{cmd_name} — Show a help message for the specified command. If no command is specified, show the descriptions of all commands.

Usage: /{cmd_name} [command]", 
                cmd_name = variant_to_string!(Command::Help)
            ),
            AddTraveler { name: _ } => format!(
"/{cmd_name} — Add a traveler with the specified name to the travel plan.

Usage: /{cmd_name} <name>",
                cmd_name = variant_to_string!(Command::AddTraveler)
            ),
            DeleteTraveler { name: _ } => format!(
"/{cmd_name} — Delete the traveler with the specified name from the travel plan.

Usage: /{cmd_name} <name>",
                cmd_name = variant_to_string!(Command::DeleteTraveler)
            ),
            ListTravelers => format!(
"/{cmd_name} — Show the travelers in the travel plan.

Usage: /{cmd_name}",
                cmd_name = variant_to_string!(Command::ListTravelers)
            ),
            AddExpense => format!(
"/{cmd_name} — Start a new interactive session to add an expense to the travel plan.

In the session, you will be asked to:
- Send a message with the description of the expense.
- Send a message with the amount of the expense.
- Send a message with the name of the traveler who paid the expense.
- Send a message with the travelers who share the expense and the amount they share.

The process can be interrupted at any time by sending `/{cancel}`. 

To split the expense among multiple travelers you can:
- Send a message for each traveler you want to share the expense with, or specify multiple travelers separating them by `{SPLIT_AMONG_ENTRIES_SEP}`.
- Use the format `<name>{SPLIT_AMONG_NAME_AMOUNT_SEP} <amount>` where `<amount>` can be followed by `%` if it is a percentage of the residual amount.
> Example: `Alice{SPLIT_AMONG_NAME_AMOUNT_SEP} 50`, `Bob{SPLIT_AMONG_NAME_AMOUNT_SEP} 20%`, `Charles`, `John{SPLIT_AMONG_NAME_AMOUNT_SEP} 30{SPLIT_AMONG_ENTRIES_SEP} Jane{SPLIT_AMONG_NAME_AMOUNT_SEP} 10%` are all valid syntaxes.
> Example: If the total is `100`, typing `Alice{SPLIT_AMONG_NAME_AMOUNT_SEP} 40{SPLIT_AMONG_ENTRIES_SEP} Bob{SPLIT_AMONG_NAME_AMOUNT_SEP} 40%{SPLIT_AMONG_ENTRIES_SEP} Charles{SPLIT_AMONG_NAME_AMOUNT_SEP} 60%` means that Alice will pay `40` so the residual is `60`, Bob will pay `24` (i.e. 40% of 60) and Charles will pay `36` (i.e. 60% of 60).

- Omit `{SPLIT_AMONG_NAME_AMOUNT_SEP} <amount>` if you want to evenly split the residual expense among the travelers.
> Example: If the total is `100`, the input `Alice{SPLIT_AMONG_NAME_AMOUNT_SEP} 40{SPLIT_AMONG_ENTRIES_SEP} Bob{SPLIT_AMONG_NAME_AMOUNT_SEP} 40%{SPLIT_AMONG_ENTRIES_SEP} Charles{SPLIT_AMONG_ENTRIES_SEP} David` is equivalent to set both Charles and David amounts to 30%.

- Enter `{ALL_KWORD}` to split it evenly among all travelers.

Usage: /{cmd_name}",
                cmd_name = variant_to_string!(Command::AddExpense),
                cancel = variant_to_string!(Command::Cancel),
            ),
            DeleteExpense { number: _ } => format!(
"/{cmd_name} — Delete the expense with the specified identifying number from the travel plan.

Usage: /{cmd_name} <number>",
                cmd_name = variant_to_string!(Command::DeleteExpense)
            ),
            ListExpenses => format!(
"/{cmd_name} — Show the expenses in the travel plan.

Usage: /{cmd_name}",
                cmd_name = variant_to_string!(Command::ListExpenses)
            ),
            FindExpenses { description: _ } => format!(
"/{cmd_name} — Search for expenses that match the given description. Supports fuzzy search for more flexible matching.

Usage: /{cmd_name} <description>",
                cmd_name = variant_to_string!(Command::FindExpenses)
            ),
            ShowExpense { number: _ } => format!(
"/{cmd_name} — Show the details of the expense with the specified identifying number.

Usage: /{cmd_name} <number>",
                cmd_name = variant_to_string!(Command::ShowExpense)
            ),
            Transfer { from: _, to: _, amount: _ } => format!(
"/{cmd_name} — Transfer the specified amount from one traveler to another.

Usage: /{cmd_name} <sender> <receiver> <amount>",
                cmd_name = variant_to_string!(Command::Transfer)
            ),
            ShowBalance { name: _ } => format!(
"/{cmd_name} — Show the simplified balance of the specified traveler.

Usage: /{cmd_name} <name>",
                cmd_name = variant_to_string!(Command::ShowBalance)
            ),
            ShowBalances => format!(
"/{cmd_name} — Show the simplified balances of all travelers.

Usage: /{cmd_name}",
                cmd_name = variant_to_string!(Command::ShowBalances)
            ),
            Cancel => format!(
"/{cmd_name} — Cancel the currently running interactive command.

Usage: /{cmd_name}",
                cmd_name = variant_to_string!(Command::Cancel)
            ),
        }
    }
}

pub async fn commands_handler(bot: Bot, msg: Message, cmd: Command) -> HandlerResult {
    use Command::*;

    let result = match cmd.clone() {
        Help { command } => help(&msg, &command),
        AddTraveler { name } => add_traveler(&msg, name).await,
        DeleteTraveler { name } => delete_traveler(&msg, name).await,
        ListTravelers => list_travelers(&msg).await,
        DeleteExpense { number } => delete_expense(&msg, number).await,
        ListExpenses => list_expenses(&msg).await,
        FindExpenses { description } => find_expenses(&msg, &description).await,
        ShowExpense { number } => show_expense(&msg, number).await,
        Transfer { from, to, amount } => transfer(&msg, from, to, amount).await,
        ShowBalance { name } => show_balance(&msg, name).await,
        ShowBalances => show_balances(&msg).await,
        Cancel | AddExpense => {
            unreachable!("This command is handled before calling this function.")
        }
    };

    match result {
        Ok(reply) => {
            bot.send_message(msg.chat.id, reply).await?;
        }
        Err(err) => {
            let err_msg = format!("{err}\n\n{help_message}", help_message = cmd.help_message());
            bot.send_message(msg.chat.id, err_msg).await?;
        }
    }

    Ok(())
}
