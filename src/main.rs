use {
    add_expense_dialogue::AddExpenseState,
    command::*,
    config::Config,
    db::init_db,
    dptree::case,
    macro_rules_attribute::attribute_alias,
    std::sync::LazyLock,
    teloxide::{dispatching::dialogue::InMemStorage, prelude::*},
    tracing_subscriber::{
        fmt::time::LocalTime, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
    },
};

mod add_expense_dialogue;
mod command;
mod db;
mod error;
mod expense;
mod traveler;

attribute_alias! {
    #[apply(trace_command)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip_all,
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from().unwrap().id
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_command_no_err)] =
    #[tracing::instrument(
        ret(level = Level::DEBUG),
        skip_all,
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from().unwrap().id
        )
    )
    ];
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap() // panics if configurations cannot be loaded
});

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_layer = tracing_subscriber::fmt::layer()
        .with_timer(LocalTime::rfc_3339())
        .with_line_number(true)
        .compact();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(log_layer)
        .init();

    tracing::info!("Starting TravelRS bot...");
    let token = CONFIG.get::<String>("token").unwrap();
    let bot = Bot::new(token);
    init_db().await?;

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .branch(
                    case![Command::AddExpense]
                        .enter_dialogue::<Message, InMemStorage<AddExpenseState>, AddExpenseState>()
                        .branch(
                            dptree::case![AddExpenseState::Start]
                                .endpoint(add_expense_dialogue::start),
                        ),
                )
                .branch(
                    case![Command::Cancel]
                        .enter_dialogue::<Message, InMemStorage<AddExpenseState>, AddExpenseState>()
                        .endpoint(add_expense_dialogue::cancel),
                )
                .branch(dptree::endpoint(commands_handler)),
        )
        .branch({
            use AddExpenseState::*;
            dptree::entry()
                .enter_dialogue::<Message, InMemStorage<AddExpenseState>, AddExpenseState>()
                .branch(
                    dptree::case![ReceiveDescription]
                        .endpoint(add_expense_dialogue::receive_description),
                )
                .branch(
                    dptree::case![ReceiveAmount { description }]
                        .endpoint(add_expense_dialogue::receive_amount),
                )
                .branch(
                    dptree::case![ReceivePayedBy {
                        description,
                        amount
                    }]
                    .endpoint(add_expense_dialogue::receive_payed_by),
                )
                .branch(
                    dptree::case![ReceiveSplitAmong {
                        description,
                        amount,
                        payed_by
                    }]
                    .endpoint(add_expense_dialogue::receive_split_among),
                )
                .branch(
                    dptree::case![End {
                        description,
                        amount,
                        payed_by,
                        split_among
                    }]
                    .endpoint(add_expense_dialogue::end),
                )
        });

    Dispatcher::builder(bot, handler)
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![InMemStorage::<AddExpenseState>::new()])
        .build()
        .dispatch()
        .await;

    Ok(())
}
