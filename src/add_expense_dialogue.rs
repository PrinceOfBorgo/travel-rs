use {
    crate::{trace_command, traveler::Traveler, HandlerResult},
    macro_rules_attribute::apply,
    rust_decimal::Decimal,
    std::collections::BTreeMap,
    teloxide::{
        dispatching::dialogue::InMemStorage, prelude::Dialogue, requests::Requester,
        types::Message, Bot,
    },
    tracing::Level,
};

type AddExpenseDialogue = Dialogue<AddExpenseState, InMemStorage<AddExpenseState>>;

#[derive(Debug, Clone, Default)]
pub enum AddExpenseState {
    #[default]
    Start,
    ReceiveDescription,
    ReceiveAmount {
        description: String,
    },
    ReceivePayedBy {
        description: String,
        amount: Decimal,
    },
    ReceiveSplitAmong {
        description: String,
        amount: Decimal,
        payed_by: Box<Traveler>,
    },
    End {
        description: String,
        amount: Decimal,
        payed_by: Box<Traveler>,
        split_among: BTreeMap<Box<Traveler>, Decimal>,
    },
}

#[apply(trace_command)]
pub async fn start(bot: Bot, dialogue: AddExpenseDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "The process can be interrupt at any time by sending /cancel.\nHow would you describe this expense?",
    )
    .await?;
    dialogue.update(AddExpenseState::ReceiveDescription).await?;
    Ok(())
}

#[apply(trace_command)]
pub async fn receive_description(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "How much is the expense?")
                .await?;
            dialogue
                .update(AddExpenseState::ReceiveAmount {
                    description: text.to_owned(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

#[apply(trace_command)]
pub async fn receive_amount(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    description: String, // Available from `AddExpenseState::ReceiveAmount`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<Decimal>()) {
        Some(Ok(amount)) => {
            bot.send_message(msg.chat.id, "Who payed for this?").await?;
            dialogue
                .update(AddExpenseState::ReceivePayedBy {
                    description,
                    amount,
                })
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Send me a number.").await?;
        }
    }

    Ok(())
}

#[apply(trace_command)]
pub async fn receive_payed_by(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    (description, amount): (String, Decimal), // Available from `AddExpenseState::ReceivePayedBy`.
    msg: Message,
) -> HandlerResult {
    Ok(())
}

#[apply(trace_command)]
pub async fn receive_split_among(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    (description, amount, payed_by): (String, Decimal, Box<Traveler>), // Available from `AddExpenseState::ReceiveSplitAmong`.
    msg: Message,
) -> HandlerResult {
    Ok(())
}

#[apply(trace_command)]
pub async fn end(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    (description, amount, payed_by, split_among): (
        String,
        Decimal,
        Box<Traveler>,
        BTreeMap<Box<Traveler>, Decimal>,
    ), // Available from `AddExpenseState::End`.
    msg: Message,
) -> HandlerResult {
    Ok(())
}

#[apply(trace_command)]
pub async fn cancel(bot: Bot, dialogue: AddExpenseDialogue, msg: Message) -> HandlerResult {
    if dialogue.get().await?.is_some() {
        dialogue.exit().await?;
        bot.send_message(msg.chat.id, "The process was cancelled.")
            .await?;
    }
    Ok(())
}
