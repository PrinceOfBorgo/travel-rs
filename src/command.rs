use {
    crate::{
        consts::{ALL_KWORD, SPLIT_AMONG_ENTRIES_SEP, SPLIT_AMONG_NAME_AMOUNT_SEP},
        errors::CommandError,
        expense::Expense,
        trace_command,
        transferred_to::TransferredTo,
        traveler::{Name, Traveler},
        update_debts, HandlerResult,
    },
    macro_rules_attribute::apply,
    rust_decimal::Decimal,
    std::sync::LazyLock,
    strum::{AsRefStr, EnumIter, IntoEnumIterator},
    teloxide::{prelude::*, utils::command::BotCommands},
    tracing::Level,
};

pub static COMMANDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    Command::iter()
        .map(|variant| variant.as_ref().to_lowercase())
        .collect()
});

pub trait HelpMessage {
    fn help_message(&self) -> String;
}

#[derive(BotCommands, Clone, EnumIter, AsRefStr)]
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
    #[command(description = "delete the expense with specified number from the travel plan.")]
    DeleteExpense { number: i64 },
    #[command(description = "show the expenses in the travel plan.")]
    ListExpenses,
    #[command(description = "find the expenses matching the specified description.")]
    FindExpenses { description: String },
    #[command(
        description = "transfer the specified amount to the specified traveler.",
        parse_with = "split"
    )]
    Transfer {
        from: Name,
        to: Name,
        amount: Decimal,
    },
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
            AddExpense => format!(
"Add a new expense to the travel plan.

- Send a message for each traveler you want to share the expense with, or specify multiple travelers separating them by `{SPLIT_AMONG_ENTRIES_SEP}`.
- Use the format `<name>{SPLIT_AMONG_NAME_AMOUNT_SEP} <amount>` where `<amount>` can be followed by `%` if it is a percentage of the residual amount.
> Example: `Alice{SPLIT_AMONG_NAME_AMOUNT_SEP} 50`, `Bob{SPLIT_AMONG_NAME_AMOUNT_SEP} 20%`, `Charles`, `John{SPLIT_AMONG_NAME_AMOUNT_SEP} 30{SPLIT_AMONG_ENTRIES_SEP} Jane{SPLIT_AMONG_NAME_AMOUNT_SEP} 10%` are all valid syntaxes.
> Example: If the total is `100`, typing `Alice{SPLIT_AMONG_NAME_AMOUNT_SEP} 40{SPLIT_AMONG_ENTRIES_SEP} Bob{SPLIT_AMONG_NAME_AMOUNT_SEP} 40%{SPLIT_AMONG_ENTRIES_SEP} Charles{SPLIT_AMONG_NAME_AMOUNT_SEP} 60%` means that Alice will pay `40` so the residual is `60`, Bob will pay `24` (i.e. 40% of 60) and Charles will pay `36` (i.e. 60% of 60).

- You can omit `{SPLIT_AMONG_NAME_AMOUNT_SEP} <amount>` if you want to evenly split the residual expense among the travelers.
> Example: If the total is `100`, the input `Alice{SPLIT_AMONG_NAME_AMOUNT_SEP} 40{SPLIT_AMONG_ENTRIES_SEP} Bob{SPLIT_AMONG_NAME_AMOUNT_SEP} 40%{SPLIT_AMONG_ENTRIES_SEP} Charles{SPLIT_AMONG_ENTRIES_SEP} David` is equivalent to set both Charles and David amounts to 30%.

- You can enter `{ALL_KWORD}` to split it evenly among all travelers."),
            _ => Command::descriptions().to_string(),
        }
    }
}

pub async fn commands_handler(bot: Bot, msg: Message, cmd: Command) -> HandlerResult {
    use Command::*;

    let result = match cmd {
        Help { command } => help(&msg, &command),
        AddTraveler { name } => add_traveler(&msg, name).await,
        DeleteTraveler { name } => delete_traveler(&msg, name).await,
        ListTravelers => list_travelers(&msg).await,
        DeleteExpense { number } => delete_expense(&msg, number).await,
        ListExpenses => list_expenses(&msg).await,
        FindExpenses { description } => find_expenses(&msg, &description).await,
        Transfer { from, to, amount } => transfer(&msg, from, to, amount).await,
        Cancel | AddExpense => {
            unreachable!("This command is handled before calling this function.")
        }
    };

    match result {
        Ok(reply) => {
            bot.send_message(msg.chat.id, reply).await?;
        }
        Err(err) => {
            bot.send_message(msg.chat.id, err.to_string()).await?;
        }
    }

    Ok(())
}

#[apply(trace_command)]
fn help(msg: &Message, command: &str) -> Result<String, CommandError> {
    tracing::debug!("START");
    let command = command.trim();
    if command.is_empty() {
        tracing::debug!("SUCCESS");
        return Ok(Command::descriptions().to_string());
    }

    match Command::iter()
        .find(|variant| variant.as_ref().to_lowercase() == command.trim_matches('/').to_lowercase())
        .map(|variant| variant.help_message())
    {
        Some(help) => {
            tracing::debug!("SUCCESS");
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

#[apply(trace_command)]
async fn add_traveler(msg: &Message, name: Name) -> Result<String, CommandError> {
    tracing::debug!("START");
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            tracing::warn!("Traveler {name} has already been added to the travel plan.");
            Ok(format!(
                "Traveler {name} has already been added to the travel plan."
            ))
        }
        Ok(_) => {
            // Create traveler on db
            let create_res = Traveler::db_create(msg.chat.id, &name).await;
            match create_res {
                Ok(_) => {
                    tracing::debug!("SUCCESS");
                    Ok(format!("Traveler {name} added successfully."))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::AddTraveler { name })
                }
            }
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::AddTraveler {
                name: name.to_owned(),
            })
        }
    }
}

#[apply(trace_command)]
async fn delete_traveler(msg: &Message, name: Name) -> Result<String, CommandError> {
    tracing::debug!("START");
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete traveler from db
            let delete_res = Traveler::db_delete(msg.chat.id, &name).await;
            match delete_res {
                Ok(_) => {
                    tracing::debug!("SUCCESS");
                    Ok(format!("Traveler {name} deleted successfully."))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteTraveler { name })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find traveler {name} to delete.");
            Ok(format!("Couldn't find traveler {name} to delete."))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteTraveler {
                name: name.to_owned(),
            })
        }
    }
}

#[apply(trace_command)]
async fn list_travelers(msg: &Message) -> Result<String, CommandError> {
    tracing::debug!("START");
    let list_res = Traveler::db_select(msg.chat.id).await;
    match list_res {
        Ok(travelers) => {
            let reply = if travelers.is_empty() {
                format!(
                    "No travelers found. Use `/{add_traveler} <name>` to add one.",
                    add_traveler = variant_to_string!(Command::AddTraveler)
                )
            } else {
                travelers
                    .into_iter()
                    .map(|traveler| (*traveler.name).to_owned())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!("SUCCESS");
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListTravelers)
        }
    }
}

#[apply(trace_command)]
async fn delete_expense(msg: &Message, number: i64) -> Result<String, CommandError> {
    tracing::debug!("START");

    // Check if expense exists on db
    let count_res = Expense::db_count(msg.chat.id, number).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete expense from db
            let delete_res = Expense::db_delete(msg.chat.id, number).await;
            match delete_res {
                Ok(_) => {
                    tracing::debug!("SUCCESS");
                    Ok(format!("Expense #{number} deleted successfully."))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteExpense { number })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find expense #{number} to delete.");
            Ok(format!("Couldn't find expense #{number} to delete."))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteExpense { number })
        }
    }
}

#[apply(trace_command)]
async fn list_expenses(msg: &Message) -> Result<String, CommandError> {
    tracing::debug!("START");
    let list_res = Expense::db_select(msg.chat.id).await;
    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                format!(
                    "No expenses found. Use `/{add_expense}` to add one.",
                    add_expense = variant_to_string!(Command::AddExpense)
                )
            } else {
                expenses
                    .into_iter()
                    .map(|expense| format!("{expense}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!("SUCCESS");
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListExpenses)
        }
    }
}

#[apply(trace_command)]
async fn find_expenses(msg: &Message, description: &str) -> Result<String, CommandError> {
    tracing::debug!("START");
    let list_res = Expense::db_select_by_descr(msg.chat.id, description.to_owned()).await;
    match list_res {
        Ok(expenses) => {
            let reply = if expenses.is_empty() {
                format!("No expenses match the specified description (~ \"{description}\").")
            } else {
                expenses
                    .into_iter()
                    .map(|expense| format!("{expense}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!("SUCCESS");
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::FindExpenses {
                description: description.to_owned(),
            })
        }
    }
}

#[apply(trace_command)]
async fn transfer(
    msg: &Message,
    from: Name,
    to: Name,
    amount: Decimal,
) -> Result<String, CommandError> {
    tracing::debug!("START");
    if from.is_empty() || to.is_empty() {
        return Err(CommandError::EmptyInput);
    }
    let chat_id = msg.chat.id;

    // Get sender from db
    let select_from_res = Traveler::db_select_by_name(chat_id, from.clone()).await;
    match select_from_res {
        Ok(senders) if !senders.is_empty() => {
            // Get receiver from db
            let select_to_res = Traveler::db_select_by_name(chat_id, to.clone()).await;
            match select_to_res {
                Ok(recvs) if !recvs.is_empty() => {
                    // Record the new transfer on db
                    let relate_res = TransferredTo::db_relate(
                        amount,
                        senders[0].id.clone(),
                        recvs[0].id.clone(),
                    )
                    .await;
                    match relate_res {
                        Ok(Some(transfer)) => {
                            if let Err(err_update) = update_debts(chat_id).await {
                                tracing::warn!("{err_update}");
                            }
                            tracing::debug!("SUCCESS - id: {}", transfer.id);
                            Ok(String::from("Transfer recorded successfully."))
                        }
                        Ok(None) => {
                            tracing::warn!("Couldn't record the transfer.");
                            Err(CommandError::Transfer {
                                from: from.to_owned(),
                                to: to.to_owned(),
                                amount,
                            })
                        }
                        Err(err) => {
                            tracing::error!("{err}");
                            Err(CommandError::Transfer {
                                from: from.to_owned(),
                                to: to.to_owned(),
                                amount,
                            })
                        }
                    }
                }
                Ok(_) => {
                    tracing::warn!("Couldn't find traveler \"{to}\" to transfer money to.");
                    Ok(format!(
                        "Couldn't find traveler \"{to}\" to transfer money to."
                    ))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::Transfer {
                        from: from.to_owned(),
                        to: to.to_owned(),
                        amount,
                    })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find traveler \"{from}\" to transfer money from.");
            Ok(format!(
                "Couldn't find traveler \"{from}\" to transfer money from."
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::Transfer {
                from: from.to_owned(),
                to: to.to_owned(),
                amount,
            })
        }
    }
}
