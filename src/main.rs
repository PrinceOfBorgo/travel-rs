#[macro_use]
mod macros;
mod add_expense_dialogue;
mod commands;
mod config;
mod consts;
mod db;
mod debt;
mod errors;
mod relationships;
mod tables;
mod utils;

pub(crate) use relationships::*;
pub(crate) use tables::*;

use std::sync::Arc;

use add_expense_dialogue::AddExpenseState;
use chat::Chat;
use commands::*;
use dptree::{case, deps};
use macro_rules_attribute::apply;
use teloxide::{
    dispatching::dialogue::{InMemStorage, Storage},
    prelude::*,
};
use tracing::Level;
use tracing_subscriber::{
    fmt::time::LocalTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use utils::*;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_layer = tracing_subscriber::fmt::layer()
        .with_timer(LocalTime::rfc_3339())
        .with_line_number(true)
        .compact();

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .with(log_layer)
        .init();

    tracing::info!("Starting TravelRS bot...");
    let token = config::get_token();
    let bot = Bot::new(token);
    tracing::info!("TravelRS bot started.");
    // Initialize the database connection
    db::db().await;

    let handler = Update::filter_message()
        .map_async(update_chat_db)  // Create chat record on db if it does not exist yet or update it
        .branch(
            dptree::entry()
                // Check if a command is received...
                .filter_command::<Command>()
                // Cancel command
                .branch(
                    case![Command::Cancel]
                        .endpoint(cancel::<InMemStorage<AddExpenseState>, AddExpenseState>),
                )
                // AddExpense command -> starts a new dialogue to add an expense
                .branch(
                    case![Command::AddExpense]
                        .enter_dialogue::<Message, InMemStorage<AddExpenseState>, AddExpenseState>()
                        .branch(case![AddExpenseState::Start].endpoint(add_expense_dialogue::start))
                        .endpoint(process_already_running::<InMemStorage<AddExpenseState>, AddExpenseState>),
                )
                // Otherwise -> handle other commands
                .branch(dptree::endpoint(commands_handler)),
        )
        .branch({
            use {add_expense_dialogue::*, AddExpenseState::*};
            dptree::entry()
                // Check if the message is a response to an add expense dialogue...
                .enter_dialogue::<Message, InMemStorage<AddExpenseState>, AddExpenseState>()
                .branch(case![ReceiveDescription].endpoint(receive_description))
                .branch(case![ReceiveAmount { description }].endpoint(receive_amount))
                .branch(
                    case![ReceivePaidBy {
                        description,
                        amount
                    }]
                    .endpoint(receive_paid_by),
                )
                .branch(
                    case![StartSplitAmong {
                        description,
                        amount,
                        paid_by
                    }]
                    .endpoint(start_split_among),
                )
                .branch(
                    case![ReceiveSplitAmong {
                        description,
                        amount,
                        paid_by,
                        split_among
                    }]
                    .endpoint(receive_split_among),
                )
        })
        .map_async(unknown_command);

    Dispatcher::builder(bot, handler)
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .dependencies(deps![InMemStorage::<AddExpenseState>::new()])
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[apply(trace_skip_all)]
pub async fn process_already_running<S, D>(bot: Bot, storage: Arc<S>, msg: Message) -> HandlerResult
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    let chat_id = msg.chat.id;
    if Arc::clone(&storage).get_dialogue(chat_id).await?.is_some() {
        bot.send_message(
            chat_id,
            format!(
                "Another process is already running, please cancel it first sending /{cancel}.",
                cancel = variant_to_string!(Command::Cancel)
            ),
        )
        .await?;
    }
    Ok(())
}

#[apply(trace_skip_all)]
pub async fn update_chat_db(msg: Message) -> HandlerResult {
    if Chat::db_create(msg.chat.id).await.is_err() {
        match Chat::db_update(msg.chat.id).await {
            Ok(Some(chat)) => {
                tracing::debug!("Chat updated on db: {chat:?}")
            }
            Ok(None) => {
                tracing::error!("Error while updating chat with id: {}", msg.chat.id)
            }
            Err(err) => tracing::error!("{err}"),
        }
    } else {
        tracing::debug!("Chat with id: {} created on db", msg.chat.id);
    }
    Ok(())
}
