#[macro_use]
mod macros;
mod balance;
mod commands;
mod consts;
mod db;
mod debt;
mod dialogues;
mod errors;
mod expense_details;
mod i18n;
mod money_wrapper;
mod relationships;
mod settings;
mod tables;
mod transfer;
mod utils;

use chat::Chat;
use clap::Parser;
use commands::*;
use dialogues::add_expense_dialogue::AddExpenseState;
use dialogues::*;
use dptree::{case, deps};
use macro_rules_attribute::apply;
use settings::{Logging, SETTINGS};
use std::sync::{Arc, LazyLock, Mutex};
use teloxide::{
    dispatching::{UpdateHandler, dialogue::InMemStorage},
    prelude::*,
};
use tracing::Level;
use tracing_appender::rolling::daily;
use tracing_subscriber::{
    EnvFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt,
};
use unic_langid::LanguageIdentifier;
use utils::*;

pub(crate) use relationships::*;
pub(crate) use tables::*;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct Context {
    langid: LanguageIdentifier,
    currency: String,
}

#[derive(Parser, Debug)]
#[command(name = "TravelRS Bot")]
pub struct Args {
    /// Profile to use
    #[arg(short, long)]
    profile: Option<String>,
}

pub static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logs
    let Logging {
        path,
        file_name_prefix,
        level,
    } = &SETTINGS.logging;

    // Initialize tracing subscriber to write logs to a file
    let file_appender = daily(path, file_name_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let log_layer = tracing_subscriber::fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .with_line_number(true)
        .compact()
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(EnvFilter::new(format!(
            "{}={level}",
            env!("CARGO_PKG_NAME").replace("-", "_"),
        )))
        .with(log_layer)
        .init();

    // Start the bot
    start_bot().await;

    Ok(())
}

async fn start_bot() {
    tracing::info!("Starting TravelRS bot...");
    let token = SETTINGS.token_value();
    let bot = Bot::new(token);
    tracing::info!("TravelRS bot started.");
    // Initialize the database connection
    db::db().await;

    Dispatcher::builder(bot, handler_tree())
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .dependencies(deps![
            InMemStorage::<AddExpenseState>::new(),
            Arc::new(Mutex::new(Context {
                langid: SETTINGS.i18n.default_locale.clone(),
                currency: SETTINGS.i18n.default_currency.clone()
            }))
        ])
        .build()
        .dispatch()
        .await;
}

fn handler_tree() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    Update::filter_message()
        .map_async(update_chat_db) // Create chat record on db if it does not exist yet or update it
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
                        .endpoint(
                            handle_process_already_running::<
                                InMemStorage<AddExpenseState>,
                                AddExpenseState,
                            >,
                        ),
                )
                // Otherwise -> handle other commands
                .endpoint(commands_handler),
        )
        .branch({
            use {AddExpenseState::*, add_expense_dialogue::*};
            dptree::entry()
                // Check if a process is running, otherwise skip the branch...
                .filter_async(
                    process_already_running::<InMemStorage<AddExpenseState>, AddExpenseState>,
                )
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
        .endpoint(unknown_command)
}

#[apply(trace_skip_all)]
pub async fn update_chat_db(msg: Message, ctx: Arc<Mutex<Context>>) -> HandlerResult {
    let mut chat = Chat::db_create(
        msg.chat.id,
        &SETTINGS.i18n.default_locale,
        &SETTINGS.i18n.default_currency,
    )
    .await;
    if chat.is_err() {
        chat = Chat::db_update_last_interaction_utc(msg.chat.id).await;
        match chat {
            Ok(Some(ref chat)) => {
                tracing::debug!("Chat updated on db: {chat:?}")
            }
            Ok(None) => {
                tracing::error!("Error while updating chat with id: {}", msg.chat.id)
            }
            Err(ref err) => tracing::error!("{err}"),
        }
    } else {
        tracing::debug!("Chat with id: {} created on db", msg.chat.id);
    }

    if let Ok(Some(chat)) = chat {
        let mut ctx = ctx.lock().expect("Failed to lock context");
        ctx.langid = chat
            .lang
            .parse()
            .unwrap_or(SETTINGS.i18n.default_locale.clone());
        ctx.currency = chat.currency;
    }

    Ok(())
}

