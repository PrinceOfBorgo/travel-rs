use {
    crate::{
        db::db,
        error::Error,
        trace_command, trace_command_no_err,
        traveler::{Name, Traveler},
        HandlerResult,
    },
    macro_rules_attribute::apply,
    teloxide::{prelude::*, utils::command::BotCommands},
    tracing::Level,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "add a traveler to the travel plan.")]
    AddTraveler { name: Name },
    #[command(description = "delete a traveler from the travel plan.")]
    DeleteTraveler { name: Name },
    #[command(description = "show the travelers in the travel plan.")]
    ListTravelers,
    #[command(
        description = "add a new expense to the travel plan, evenly split among the travelers."
    )]
    AddExpense,
    #[command(
        description = "add a new expense to the travel plan, unevenly split among the travelers. The amount each traveler has to pay can be specified as a fixed value or with a percentage of the total, using %."
    )]
    AddUnevenExpense,
    #[command(description = "delete the expense with specified ID from the travel plan.")]
    DeleteExpense { expense_id: usize },
    #[command(description = "show the expenses in the travel plan.")]
    ListExpenses,
    #[command(description = "cancel the current interactive command (e.g. /add_expense).")]
    Cancel,
}

pub async fn commands_handler(bot: Bot, msg: Message, cmd: Command) -> HandlerResult {
    use Command::*;

    let result = match cmd {
        Help => Ok(help(&msg)),
        AddTraveler { name } => add_traveler(&msg, name).await,
        DeleteTraveler { name } => delete_traveler(&msg, name).await,
        ListTravelers => list_travelers(&msg).await,
        AddExpense => add_expense(&msg, false).await,
        AddUnevenExpense => add_expense(&msg, true).await,
        DeleteExpense { expense_id } => delete_expense(&msg, expense_id).await,
        ListExpenses => list_expenses(&msg).await,
        Cancel => Ok(String::new()),
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

#[apply(trace_command_no_err)]
fn help(msg: &Message) -> String {
    Command::descriptions().to_string()
}

#[apply(trace_command)]
async fn add_traveler(msg: &Message, name: Name) -> Result<String, Error> {
    tracing::debug!("START");
    if name.is_empty() {
        return Err(Error::EmptyInput);
    }

    let db = db().await;
    let result: Result<Vec<Traveler>, surrealdb::Error> = db
        .create("traveler")
        .content(Traveler {
            chat_id: msg.chat.id,
            name: name.to_owned(),
        })
        .await;

    match result {
        Ok(_) => {
            tracing::debug!("SUCCESS");
            Ok(format!("Traveler {name} added successfully."))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(Error::AddTraveler { name })
        }
    }
}

#[apply(trace_command)]
async fn delete_traveler(msg: &Message, name: Name) -> Result<String, Error> {
    tracing::debug!("START");
    if name.is_empty() {
        return Err(Error::EmptyInput);
    }

    let db = db().await;
    let result = db
        .query("DELETE traveler WHERE chat_id = $chat_id && name = $name")
        .bind(Traveler {
            chat_id: msg.chat.id,
            name: name.to_owned(),
        })
        .await;

    match result {
        Ok(_) => {
            tracing::debug!("SUCCESS");
            Ok(format!("Traveler {name} deleted successfully."))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(Error::DeleteTraveler {
                name: name.to_owned(),
            })
        }
    }
}

#[apply(trace_command)]
async fn list_travelers(msg: &Message) -> Result<String, Error> {
    tracing::debug!("START");
    let db = db().await;
    let result = db
        .query("SELECT * FROM traveler WHERE chat_id = $chat_id")
        .bind(("chat_id", msg.chat.id))
        .await
        .and_then(|mut response| response.take::<Vec<Traveler>>(0));

    match result {
        Ok(travelers) => {
            let reply = if travelers.is_empty() {
                String::from("No travelers found. Use /addtraveler <name> to add one.")
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
            Err(Error::ListTravelers)
        }
    }
}

#[apply(trace_command)]
async fn add_expense(msg: &Message, uneven: bool) -> Result<String, Error> {
    let reply = format!("add_expense({uneven}): TODO");
    Ok(reply)
}

#[apply(trace_command)]
async fn delete_expense(msg: &Message, expense_id: usize) -> Result<String, Error> {
    let reply = format!("delete_expense({expense_id}): TODO");
    Ok(reply)
}

#[apply(trace_command)]
async fn list_expenses(msg: &Message) -> Result<String, Error> {
    let reply = "list_expenses(): TODO".to_string();
    Ok(reply)
}
