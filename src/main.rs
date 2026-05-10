// `surrealdb::Error` is ~144 bytes, which trips clippy's `result_large_err` lint
// for every function returning `Result<_, surrealdb::Error>`. Boxing the error
// would require touching dozens of public signatures and all call sites, so we
// allow the lint crate-wide instead.
#![allow(clippy::result_large_err)]

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

mod balance;
mod commands;
mod consts;
mod db;
mod debt;
mod dialogues;
mod errors;
mod expense_details;
mod i18n;
pub(crate) mod keyboard;
mod money_wrapper;
mod relationships;
mod settings;
mod stats;
mod tables;
mod transfer;

use chat::Chat;
use clap::Parser;
use commands::*;
use dialogues::add_expense_dialogue::{self, AddExpenseState};
use dialogues::pending_command_dialogue::{
    self, PendingCommandState, PendingCommandStorage,
    add_traveler::{self as pending_add_traveler},
    delete_expense::{self as pending_delete_expense},
    delete_transfer::{self as pending_delete_transfer},
    delete_traveler::{self as pending_delete_traveler},
    list_expenses::{self as pending_list_expenses},
    set_currency::{self as pending_set_currency},
    set_language::{self as pending_set_language},
    show_expense::{self as pending_show_expense},
    transfer::{self as pending_transfer},
};
use dialogues::storage::{self as dialogue_storage, DialogueRegistry, DialogueStorages};
use dptree::{case, deps};
use macro_rules_attribute::apply;
use settings::{Logging, SETTINGS};
use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    dispatching::{UpdateHandler, dialogue::InMemStorage},
    prelude::*,
    types::{BotCommandScope, Recipient},
};
use tracing::Level;
use tracing_appender::rolling::daily;
use tracing_subscriber::{
    EnvFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt,
};
use unic_langid::LanguageIdentifier;

use debt::update_debts;

pub(crate) use relationships::*;
pub(crate) use tables::*;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct Context {
    langid: LanguageIdentifier,
    currency: String,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            langid: SETTINGS.i18n.default_locale.clone(),
            currency: SETTINGS.i18n.default_currency.clone(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "TravelRS Bot")]
pub struct Args {
    /// Profile to use
    #[arg(short, long)]
    profile: Option<String>,
}

pub static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

/// Tracks which (chat, language) pairs have already had their localized
/// command list registered with Telegram during this process lifetime.
static REGISTERED_LOCALIZED_COMMANDS: LazyLock<Mutex<HashSet<(ChatId, String)>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const ASCII_LOG: &str = r#"
  ______                      __      ____  _____
 /_  __/________ __   _____  / /     / __ \/ ___/
  / / / ___/ __ `/ | / / _ \/ /_____/ /_/ /\__ \ 
 / / / /  / /_/ /| |/ /  __/ /_____/ _, _/___/ / 
/_/ /_/_  \__,_/_|___/\___/_/     /_/ |_|/____/  
   / __ )____  / /_                              
  / __  / __ \/ __/                              
 / /_/ / /_/ / /_                                
/_____/\____/\__/                                "#;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logs
    let Logging {
        path,
        file_name_prefix,
        level,
    } = &SETTINGS.logging;

    let path = std::path::Path::new(path).join(&SETTINGS.profile);
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

    tracing::info!("{ASCII_LOG}\nv{}\n", env!("CARGO_PKG_VERSION"));
    tracing::info!("Using profile {}", SETTINGS.profile);
    tracing::debug!("Settings: {:#?}", SETTINGS);
    println!("Using profile {}", SETTINGS.profile);
    println!("Settings: {:#?}", SETTINGS);

    // Start the bot
    start_bot().await;

    Ok(())
}

async fn start_bot() {
    tracing::info!("Starting TravelRS bot...");
    let token = SETTINGS.token_value();
    let bot = Bot::new(token);
    tracing::info!("TravelRS bot started.");

    // Initialize the database connection.
    let db_instance = db::db().await;

    Dispatcher::builder(bot, handler_tree())
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .dependencies(deps(db_instance))
        .build()
        .dispatch()
        .await;
}

fn deps(db_instance: Arc<Surreal<Any>>) -> DependencyMap {
    let storages = DialogueStorages {
        add_expense: InMemStorage::<AddExpenseState>::new(),
        pending_command: PendingCommandStorage::new(),
    };
    let registry = DialogueRegistry::build(&storages);
    let DialogueStorages {
        add_expense,
        pending_command,
    } = storages;
    deps![
        add_expense,
        pending_command,
        registry,
        Arc::new(Mutex::new(Context::default())),
        db_instance
    ]
}

pub(crate) fn handler_tree() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Symmetric guard applied before every dialogue entry point: if *any*
    // known dialogue is already running for this chat, reply with
    // `process-already-running` and stop. This prevents two dialogues from
    // being alive at the same time regardless of which one is started first.
    let any_dialogue_running_guard = || {
        dptree::entry()
            .filter_async(dialogue_storage::any_running)
            .endpoint(dialogue_storage::process_already_running_endpoint)
    };

    let command_branch = dptree::entry()
        // Check if a command is received...
        .filter_command::<Command>()
        // Cancel command
        .branch(case![Command::Cancel].endpoint(cancel))
        // AddExpense command -> start a new dialogue to add an expense.
        // Refuse if any dialogue is already running.
        .branch(
            case![Command::AddExpense]
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, InMemStorage<AddExpenseState>, AddExpenseState>()
                .branch(case![AddExpenseState::Start].endpoint(add_expense_dialogue::start)),
        )
        // AddTraveler command without an inline name -> start a dialogue to
        // ask for the name. If a name was supplied inline, this branch is
        // skipped and the message falls through to `commands_handler`.
        // Refuse if any dialogue is already running.
        .branch(
            case![Command::AddTraveler { name }]
                .filter(|name: CommandArg<traveler::Name>| name.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_add_traveler::start)),
        )
        // DeleteTraveler command without an inline name -> start a dialogue
        // to ask for the name.
        .branch(
            case![Command::DeleteTraveler { name }]
                .filter(|name: CommandArg<traveler::Name>| name.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_delete_traveler::start)),
        )
        // DeleteTraveler with inline name -> start confirmation dialogue.
        .branch(
            case![Command::DeleteTraveler { name }]
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(
                    case![PendingCommandState::Start]
                        .endpoint(pending_delete_traveler::start_confirm),
                ),
        )
        // DeleteExpense without an inline number -> start dialogue.
        .branch(
            case![Command::DeleteExpense { number }]
                .filter(|number: CommandArg<i64>| number.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_delete_expense::start)),
        )
        // DeleteExpense with inline number -> start confirmation dialogue.
        .branch(
            case![Command::DeleteExpense { number }]
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(
                    case![PendingCommandState::Start]
                        .endpoint(pending_delete_expense::start_confirm),
                ),
        )
        // ShowExpense without an inline number -> start dialogue.
        .branch(
            case![Command::ShowExpense { number }]
                .filter(|number: CommandArg<i64>| number.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_show_expense::start)),
        )
        // DeleteTransfer without an inline number -> start dialogue.
        .branch(
            case![Command::DeleteTransfer { number }]
                .filter(|number: CommandArg<i64>| number.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_delete_transfer::start)),
        )
        // DeleteTransfer with inline number -> start confirmation dialogue.
        .branch(
            case![Command::DeleteTransfer { number }]
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(
                    case![PendingCommandState::Start]
                        .endpoint(pending_delete_transfer::start_confirm),
                ),
        )
        // SetLanguage without an inline langid -> start dialogue.
        .branch(
            case![Command::SetLanguage { langid }]
                .filter(|langid: CommandArg<LanguageIdentifier>| langid.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_set_language::start)),
        )
        // SetCurrency without an inline currency -> start dialogue.
        .branch(
            case![Command::SetCurrency { currency }]
                .filter(|currency: CommandArg<String>| currency.is_missing())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_set_currency::start)),
        )
        // Transfer without inline args -> start multi-step dialogue.
        .branch(
            case![Command::Transfer { args }]
                .filter(|args: String| args.is_empty())
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(case![PendingCommandState::Start].endpoint(pending_transfer::start)),
        )
        // Transfer with 1 arg (from only) -> start dialogue at AskTo.
        .branch(
            case![Command::Transfer { args }]
                .filter(|args: String| args.split_whitespace().count() == 1)
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(
                    case![PendingCommandState::Start].endpoint(pending_transfer::start_with_from),
                ),
        )
        // Transfer with 2 args (from + to) -> start dialogue at AskAmount.
        .branch(
            case![Command::Transfer { args }]
                .filter(|args: String| args.split_whitespace().count() == 2)
                .branch(any_dialogue_running_guard())
                .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
                .branch(
                    case![PendingCommandState::Start]
                        .endpoint(pending_transfer::start_with_from_to),
                ),
        )
        // Otherwise -> handle other commands
        .endpoint(commands_handler);

    let pending_command_dialogue_branch = pending_command_dialogue::handler_branch();

    let add_expense_dialogue_branch = add_expense_dialogue::handler_branch();

    let message_branch = Update::filter_message()
        .filter(|msg: Message| filter_auth(msg))
        .map_async(update_chat_db) // Create chat record on db if it does not exist yet or update it
        .branch(command_branch)
        .branch(add_expense_dialogue_branch)
        .branch(pending_command_dialogue_branch)
        .endpoint(unknown_command);

    // Inline-keyboard callback queries for pending-command dialogues.
    let dialogue_callback_branch = Update::filter_callback_query()
        .filter(|q: CallbackQuery| {
            q.data
                .as_deref()
                .is_some_and(pending_command_dialogue::is_pending_callback)
        })
        .filter(|q: CallbackQuery| {
            q.regular_message()
                .map(|m| is_chat_whitelisted(m.chat.id))
                .unwrap_or(false)
        })
        .branch(pending_command_dialogue::callback_branch());

    // Stateless command-keyboard callbacks.
    // These don't require a dialogue — the callback data is self-contained.
    let stateless_callback_branch = Update::filter_callback_query()
        .filter(|q: CallbackQuery| q.data.as_deref().is_some_and(is_stateless_callback))
        .filter(|q: CallbackQuery| {
            q.regular_message()
                .map(|m| is_chat_whitelisted(m.chat.id))
                .unwrap_or(false)
        })
        .endpoint(stateless_callback_endpoint);

    // `/listexpenses` "Filter…" callback: starts a pending-command dialogue
    // to ask for the description.
    let list_expenses_filter_callback_branch = Update::filter_callback_query()
        .filter(|q: CallbackQuery| {
            q.data
                .as_deref()
                .is_some_and(|d| d == LIST_EXPENSES_FILTER_CALLBACK)
        })
        .filter(|q: CallbackQuery| {
            q.regular_message()
                .map(|m| is_chat_whitelisted(m.chat.id))
                .unwrap_or(false)
        })
        .endpoint(pending_list_expenses::receive_filter_callback);

    // AddExpense payer-picker and split-picker callbacks. Uses its own
    // dialogue storage (InMemStorage<AddExpenseState>) so it needs a separate branch.
    let add_expense_callback_branch = Update::filter_callback_query()
        .filter(|q: CallbackQuery| {
            q.data
                .as_deref()
                .is_some_and(add_expense_dialogue::is_add_expense_callback)
        })
        .filter(|q: CallbackQuery| {
            q.regular_message()
                .map(|m| is_chat_whitelisted(m.chat.id))
                .unwrap_or(false)
        })
        .branch(add_expense_dialogue::callback_branch());

    dptree::entry()
        .branch(message_branch)
        .branch(dialogue_callback_branch)
        .branch(add_expense_callback_branch)
        .branch(stateless_callback_branch)
        .branch(list_expenses_filter_callback_branch)
}

fn filter_auth(msg: Message) -> bool {
    is_chat_whitelisted(msg.chat.id)
}

fn is_chat_whitelisted(chat_id: ChatId) -> bool {
    let whitelist = &SETTINGS.chat_whitelist_value();

    if !whitelist.is_empty() && !whitelist.contains(&chat_id) {
        tracing::warn!(
            "A non-empty whitelist is set, but the chat with id {} is not whitelisted. Skipping...",
            chat_id
        );
        false
    } else {
        true
    }
}

#[apply(trace_skip_all)]
async fn update_chat_db(
    bot: Bot,
    db: Arc<Surreal<Any>>,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    let mut chat_res = Chat::db_select_by_id(db.clone(), msg.chat.id).await;
    match chat_res {
        Ok(Some(ref chat)) => {
            tracing::debug!("Chat found on db: {chat:?}");

            chat_res = Chat::db_update_last_interaction_utc(db, msg.chat.id).await;
            match chat_res {
                Ok(Some(ref chat)) => {
                    tracing::debug!("Chat updated on db: {chat:?}")
                }
                Ok(None) => {
                    tracing::error!("Error while updating chat with id: {}", msg.chat.id)
                }
                Err(ref err) => tracing::error!("{err}"),
            };
        }
        Ok(None) => {
            tracing::debug!("Chat with id {} not found on db. Creating it.", msg.chat.id);

            chat_res = Chat::db_create(
                db,
                msg.chat.id,
                &SETTINGS.i18n.default_locale,
                &SETTINGS.i18n.default_currency,
            )
            .await;

            match chat_res {
                Ok(Some(_)) => {
                    tracing::debug!("Chat with id: {} created on db", msg.chat.id);
                }
                Ok(None) => {
                    tracing::error!("Error while creating chat with id: {}", msg.chat.id)
                }
                Err(ref err) => tracing::error!("{err}"),
            };
        }
        Err(ref err) => {
            tracing::error!("{err}");
        }
    }

    if let Ok(Some(chat)) = chat_res {
        let langid = {
            let mut ctx_guard = ctx.lock().expect("Failed to lock context");
            ctx_guard.langid = chat
                .lang
                .parse()
                .unwrap_or(SETTINGS.i18n.default_locale.clone());
            ctx_guard.currency = chat.currency;
            ctx_guard.langid.clone()
        };

        register_localized_commands(&bot, msg.chat.id, &langid, ctx).await;
    }

    Ok(())
}

async fn register_localized_commands(
    bot: &Bot,
    chat_id: ChatId,
    langid: &LanguageIdentifier,
    ctx: Arc<Mutex<Context>>,
) {
    let key = (chat_id, langid.to_string());
    {
        let mut registered = REGISTERED_LOCALIZED_COMMANDS
            .lock()
            .expect("Failed to lock REGISTERED_LOCALIZED_COMMANDS");
        if registered.contains(&key) {
            return;
        }
        registered.insert(key.clone());
    }

    let translated = Command::localized_bot_commands(ctx);
    if let Err(err) = bot
        .set_my_commands(translated)
        .scope(BotCommandScope::Chat {
            chat_id: Recipient::Id(chat_id),
        })
        .await
    {
        tracing::error!("Failed setting bot commands for chat {chat_id}: {err}");
        // Roll back so a future message retries.
        REGISTERED_LOCALIZED_COMMANDS
            .lock()
            .expect("Failed to lock REGISTERED_LOCALIZED_COMMANDS")
            .remove(&key);
    }
}
