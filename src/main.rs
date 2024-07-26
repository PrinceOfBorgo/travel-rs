#[macro_use]
mod macros;
mod add_expense_dialogue;
mod chat;
mod command;
mod consts;
mod db;
mod errors;
mod expense;
mod paid_for;
mod split;
mod traveler;
mod utils;

use {
    add_expense_dialogue::AddExpenseState,
    chat::Chat,
    command::*,
    config::Config,
    dptree::{case, deps},
    macro_rules_attribute::apply,
    std::sync::{Arc, LazyLock},
    teloxide::{
        dispatching::dialogue::{InMemStorage, Storage},
        prelude::*,
    },
    tracing::Level,
    tracing_subscriber::{
        fmt::time::LocalTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
    },
    utils::*,
};

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap() // Panics if configurations cannot be loaded
});

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
    let token = CONFIG.get::<String>("token").unwrap();
    let bot = Bot::new(token);
    tracing::info!("TravelRS bot started.");
    // Initialize the database connection
    db::db().await;

    let handler = Update::filter_message()
        // .chain(
        //     // Create chat record on db if it does not exist yet or update it
        //     dptree::endpoint(update_chat_db).map(|| {
        //         tracing::debug!("Continue control flow...");
        //         // Always return ControlFlow::Continue here to continue to the next branch
        //         ControlFlow::<Result<(), HandlerResult>>::Continue
        //     }),
        // )
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
        });

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
pub async fn cancel<S, D>(bot: Bot, storage: Arc<S>, msg: Message) -> HandlerResult
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: std::error::Error + Send + Sync,
    D: Default + Send + Sync + 'static,
{
    let chat_id = msg.chat.id;
    if Arc::clone(&storage).get_dialogue(chat_id).await?.is_some() {
        Dialogue::new(storage, chat_id).exit().await?;
        bot.send_message(chat_id, "The process was cancelled.")
            .await?;
    } else {
        bot.send_message(chat_id, "There is no process to cancel.")
            .await?;
    }
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
