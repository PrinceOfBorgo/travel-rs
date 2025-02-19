use crate::{
    consts::*,
    db::db,
    errors::{AddExpenseError, EndError},
    expense::Expense,
    trace_state,
    traveler::{Name, Traveler},
    update_debts, HandlerResult,
};
use macro_rules_attribute::apply;
use regex::Regex;
use rust_decimal::Decimal;
use std::{collections::BTreeMap, fmt::Debug, str::FromStr, sync::LazyLock};
use surrealdb::{
    sql::statements::{BeginStatement, CommitStatement},
    RecordId,
};
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::Dialogue,
    requests::Requester,
    types::{ChatId, Message},
    Bot,
};
use tracing::Level;

type AddExpenseDialogue = Dialogue<AddExpenseState, InMemStorage<AddExpenseState>>;
static SPLIT_AMONG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        format!(r"^\s*(?P<{SPLIT_AMONG_REGEX_NAME_GRP}>[^{name_amount_sep}]+)(\s*{name_amount_sep}\s*(?P<{SPLIT_AMONG_REGEX_AMOUNT_GRP}>\d+({decimal_sep}\d+)?\s*(?P<{SPLIT_AMONG_REGEX_PERCENTAGE_GRP}>%)?))?\s*$",
            name_amount_sep = regex::escape(&SPLIT_AMONG_NAME_AMOUNT_SEP.to_string()) ,
            decimal_sep = regex::escape(&DECIMAL_SEP.to_string())
        ).as_str()
    ).unwrap()
});

#[derive(Debug, Clone, Default)]
pub enum AddExpenseState {
    #[default]
    Start,
    ReceiveDescription,
    ReceiveAmount {
        description: String,
    },
    ReceivePaidBy {
        description: String,
        amount: Decimal,
    },
    StartSplitAmong {
        description: String,
        amount: Decimal,
        paid_by: Traveler,
    },
    ReceiveSplitAmong {
        description: String,
        amount: Decimal,
        paid_by: Traveler,
        split_among: BTreeMap<Name, AmountEnum>,
    },
}
#[derive(Debug, Clone)]
pub enum SplitAmongEnum {
    All,
    List,
    End,
}

#[derive(Debug, Clone)]
pub enum AmountEnum {
    Fixed(Decimal),
    Percentage(Decimal),
    Dynamic,
}

#[apply(trace_state)]
pub async fn start(bot: Bot, dialogue: AddExpenseDialogue, msg: Message) -> HandlerResult {
    tracing::debug!("START");
    bot.send_message(
        msg.chat.id,
        format!("The process can be interrupt at any time by sending `/{cancel}`.\nHow would you describe this expense?", 
            cancel = variant_to_string!(Command::Cancel),
        )
    )
    .await?;
    dialogue.update(AddExpenseState::ReceiveDescription).await?;
    tracing::debug!("SUCCESS");
    Ok(())
}

#[apply(trace_state)]
pub async fn receive_description(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    msg: Message,
) -> HandlerResult {
    tracing::debug!("START");
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "How much is the expense?")
                .await?;
            dialogue
                .update(AddExpenseState::ReceiveAmount {
                    description: text.to_owned(),
                })
                .await?;
            tracing::debug!("SUCCESS");
        }
        None => {
            tracing::warn!("Invalid description: received `None`.");
            bot.send_message(msg.chat.id, "You sent an invalid text, please retry.")
                .await?;
        }
    }

    Ok(())
}

#[apply(trace_state)]
pub async fn receive_amount(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    description: String, // Available from `AddExpenseState::ReceiveAmount`.
    msg: Message,
) -> HandlerResult {
    tracing::debug!("START");
    let parsed_text = msg.text().map(|text| text.parse::<Decimal>());
    match parsed_text {
        Some(Ok(amount)) => {
            bot.send_message(msg.chat.id, "Who paid for this?").await?;
            dialogue
                .update(AddExpenseState::ReceivePaidBy {
                    description,
                    amount,
                })
                .await?;
            tracing::debug!("SUCCESS");
        }
        _ => {
            tracing::warn!("Invalid amount: received `{parsed_text:?}`.");
            bot.send_message(msg.chat.id, "You sent an invalid amount, please retry.")
                .await?;
        }
    }

    Ok(())
}

#[apply(trace_state)]
pub async fn receive_paid_by(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    (description, amount): (String, Decimal), // Available from `AddExpenseState::ReceivePaidBy`.
    msg: Message,
) -> HandlerResult {
    tracing::debug!("START");
    let text = msg.text();

    let Some(name) = text.and_then(|text| Name::from_str(text).ok()) else {
        tracing::warn!("Invalid name: received `{text:?}`.");
        bot.send_message(msg.chat.id, "You sent an invalid name, please retry.")
            .await?;
        return Ok(());
    };

    // Select traveler from db
    let select_res = Traveler::db_select_by_name(msg.chat.id, name.clone()).await;
    match select_res {
        Ok(travelers) if !travelers.is_empty() => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "How would you like to split the expense? Type `/{help} {add_expense}` for more info.", 
                    help = variant_to_string!(Command::Help),
                    add_expense= variant_to_string!(Command::AddExpense)
                )
            ).await?;
            dialogue
                .update(AddExpenseState::StartSplitAmong {
                    description,
                    amount,
                    paid_by: travelers[0].clone(),
                })
                .await?;
            tracing::debug!("SUCCESS");
        }
        Ok(_) => {
            tracing::warn!("Invalid traveler: received {name}.");
            bot.send_message(msg.chat.id, format!("Couldn't find traveler {name}. Specify the traveler who paid for this expense.")).await?;
        }
        Err(err) => {
            tracing::error!("{err}");
            bot.send_message(
                msg.chat.id,
                format!("An error occured while looking for traveler {name}. Please retry."),
            )
            .await?;
        }
    }

    Ok(())
}

#[apply(trace_state)]
pub async fn start_split_among(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    (description, amount, paid_by): (String, Decimal, Traveler), // Available from `AddExpenseState::StartSplitAmong`.
    msg: Message,
) -> HandlerResult {
    tracing::debug!("START");
    match msg.text() {
        Some(text) => {
            tracing::debug!("Received text: `{text}`.");
            let split_res = parse_split_among(text, msg.chat.id, BTreeMap::new()).await;
            match split_res {
                Ok((SplitAmongEnum::All, split_among)) => {
                    tracing::debug!("SUCCESS");
                    match end(
                        dialogue,
                        (description, amount, paid_by, split_among),
                        msg.chat.id,
                    )
                    .await
                    {
                        Ok(expense) => {
                            bot.send_message(
                                msg.chat.id,
                                format!("Expense added successfully!\n{expense}"),
                            )
                            .await?;
                        }
                        Err(err) => match err {
                            EndError::ClosingDialogue => {
                                bot.send_message(
                                    msg.chat.id,
                                    "An error occured while closing the process.",
                                )
                                .await?;
                            }
                            EndError::NoExpenseCreated => {
                                bot.send_message(msg.chat.id, "No expense has been created.")
                                    .await?;
                            }
                            EndError::AddExpense(_) => {
                                bot.send_message(
                                    msg.chat.id,
                                    "An error occured while computing shares.",
                                )
                                .await?;
                            }
                            EndError::Generic(_) => {
                                bot.send_message(
                                    msg.chat.id,
                                    "An error occured while creating expense.",
                                )
                                .await?;
                            }
                        },
                    }
                }
                Ok((SplitAmongEnum::List, split_among)) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Continue splitting or type `{END_KWORD}` to end the process.",),
                    )
                    .await?;
                    dialogue
                        .update(AddExpenseState::ReceiveSplitAmong {
                            description,
                            amount,
                            paid_by,
                            split_among,
                        })
                        .await?;
                    tracing::debug!("SUCCESS");
                }
                Ok((SplitAmongEnum::End, _)) => {
                    unreachable!() // This branch already returns an error in parse_split_among
                }
                Err(err) => {
                    tracing::error!("{err}");
                    bot.send_message(
                        msg.chat.id,
                        match err {
                            AddExpenseError::Generic(_) => String::from(
                                "An error occured while parsing the text. Please retry.",
                            ),
                            _ => format!("{err}"),
                        },
                    )
                    .await?;
                }
            }
        }
        None => {
            tracing::warn!("Invalid text: received `None`.");
            bot.send_message(msg.chat.id, "You sent an invalid text, please retry.")
                .await?;
        }
    }

    Ok(())
}

#[apply(trace_state)]
pub async fn receive_split_among(
    bot: Bot,
    dialogue: AddExpenseDialogue,
    (description, amount, paid_by, split_among): (
        String,
        Decimal,
        Traveler,
        BTreeMap<Name, AmountEnum>,
    ), // Available from `AddExpenseState::ReceiveSplitAmong`.
    msg: Message,
) -> HandlerResult {
    tracing::debug!("START");
    match msg.text() {
        Some(text) => {
            tracing::debug!("Received text: `{text}`.");
            let split_res = parse_split_among(text, msg.chat.id, split_among).await;
            match split_res {
                Ok((SplitAmongEnum::All, _)) => {
                    unreachable!() // This branch already returns an error in parse_split_among
                }
                Ok((SplitAmongEnum::List, split_among)) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Continue splitting or type `{END_KWORD}` to end the process.",),
                    )
                    .await?;
                    dialogue
                        .update(AddExpenseState::ReceiveSplitAmong {
                            description,
                            amount,
                            paid_by,
                            split_among,
                        })
                        .await?;
                    tracing::debug!("SUCCESS");
                }
                Ok((SplitAmongEnum::End, split_among)) => {
                    tracing::debug!("SUCCESS");
                    match end(
                        dialogue,
                        (description, amount, paid_by, split_among),
                        msg.chat.id,
                    )
                    .await
                    {
                        Ok(expense) => {
                            bot.send_message(
                                msg.chat.id,
                                format!("Expense added successfully!\n{expense}"),
                            )
                            .await?;
                        }
                        Err(err) => match err {
                            EndError::ClosingDialogue => {
                                bot.send_message(
                                    msg.chat.id,
                                    "An error occured while closing the process.",
                                )
                                .await?;
                            }
                            EndError::NoExpenseCreated => {
                                bot.send_message(msg.chat.id, "No expense has been created.")
                                    .await?;
                            }
                            EndError::AddExpense(_) => {
                                bot.send_message(
                                    msg.chat.id,
                                    "An error occured while computing shares.",
                                )
                                .await?;
                            }
                            EndError::Generic(_) => {
                                bot.send_message(
                                    msg.chat.id,
                                    "An error occured while creating expense.",
                                )
                                .await?;
                            }
                        },
                    }
                }
                Err(err) => {
                    tracing::error!("{err}");
                    bot.send_message(
                        msg.chat.id,
                        match err {
                            AddExpenseError::Generic(_) => String::from(
                                "An error occured while parsing the text. Please retry.",
                            ),
                            _ => format!("{err}"),
                        },
                    )
                    .await?;
                }
            }
        }
        None => {
            tracing::warn!("Invalid text: received `None`.");
            bot.send_message(msg.chat.id, "You sent an invalid text, please retry.")
                .await?;
        }
    }

    Ok(())
}

#[tracing::instrument(
    err(level = Level::ERROR),
    ret(level = Level::DEBUG),
    skip_all,
)]
pub async fn end(
    dialogue: AddExpenseDialogue,
    (description, amount, paid_by, split_among): (
        String,
        Decimal,
        Traveler,
        BTreeMap<Name, AmountEnum>,
    ),
    chat_id: ChatId,
) -> Result<Expense, EndError> {
    tracing::debug!("START");
    match compute_shares(amount, split_among) {
        Ok(shares) => {
            let create_res = Expense::db_create(chat_id, description.clone(), amount).await;
            match create_res {
                Ok(Some(expense)) => {
                    if let Err(err_relate) = relate_shares(paid_by, &expense, shares).await {
                        if let Err(err_delete) = Expense::db_delete(chat_id, expense.number).await {
                            tracing::warn!("{err_delete}");
                        }
                        tracing::error!("{err_relate}");
                        Err(EndError::ClosingDialogue)
                    } else {
                        if let Err(err_update) = update_debts(chat_id).await {
                            tracing::warn!("{err_update}");
                        }
                        match dialogue.exit().await {
                            Ok(_) => {
                                tracing::debug!("SUCCESS - id: {}", expense.id);
                                Ok(expense)
                            }
                            Err(err_closing) => {
                                tracing::error!("{err_closing}");
                                Err(EndError::ClosingDialogue)
                            }
                        }
                    }
                }
                Ok(None) => {
                    tracing::error!("No expense has been created.");
                    Err(EndError::NoExpenseCreated)
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(EndError::Generic(Box::new(err)))
                }
            }
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(EndError::AddExpense(err))
        }
    }
}

async fn parse_split_among(
    text: &str,
    chat_id: ChatId,
    mut split_among: BTreeMap<Name, AmountEnum>,
) -> Result<(SplitAmongEnum, BTreeMap<Name, AmountEnum>), AddExpenseError> {
    let text = text.trim();
    let text_lower = text.to_lowercase();

    // If the user wants to end the dialogue
    if text_lower == END_KWORD.to_lowercase() {
        if !split_among.is_empty() {
            Ok((SplitAmongEnum::End, split_among))
        } else {
            Err(AddExpenseError::NoTravelersSpecified)
        }
    }
    // If the expense should be split evenly among all travelers
    else if text_lower == ALL_KWORD.to_lowercase() {
        let travelers = Traveler::db_select(chat_id)
            .await
            .map_err(|err| AddExpenseError::Generic(Box::new(err)))?;

        split_among.append(
            &mut travelers
                .into_iter()
                .filter(|traveler| !split_among.contains_key(&traveler.name))
                .map(|traveler| (traveler.name, AmountEnum::Dynamic))
                .collect(),
        );
        Ok((SplitAmongEnum::All, split_among))
    }
    // If the user specified a list of travelers
    else {
        let entries = text.split(SPLIT_AMONG_ENTRIES_SEP);
        for entry in entries {
            tracing::debug!(
                "Parsing entry: {entry} with regex: {regex}",
                regex = SPLIT_AMONG_REGEX.as_str()
            );
            let caps = SPLIT_AMONG_REGEX
                .captures(entry)
                .ok_or(AddExpenseError::InvalidFormat {
                    input: entry.to_owned(),
                })?;
            let name = Name::from_str(&caps[SPLIT_AMONG_REGEX_NAME_GRP])
                .map_err(|err| AddExpenseError::Generic(Box::new(err)))?;
            if split_among.contains_key(&name) {
                return Err(AddExpenseError::RepeatedTravelerName { name });
            }

            if let Some(amount) = caps.name(SPLIT_AMONG_REGEX_AMOUNT_GRP) {
                let amount = amount.as_str().replace(DECIMAL_SEP, "."); // Replace decimal separator with '.' so Decimal::from_str won't fail
                let amount = amount.trim_end_matches(|c: char| c.is_whitespace() || c == '%'); // Remove whitespaces and '%' at the end of the amount
                let amount = Decimal::from_str(amount).unwrap(); // Can unwrap since the regex only matches positive numbers

                if caps.name(SPLIT_AMONG_REGEX_PERCENTAGE_GRP).is_some() {
                    split_among.insert(name, AmountEnum::Percentage(amount));
                } else {
                    split_among.insert(name, AmountEnum::Fixed(amount));
                }
            } else {
                split_among.insert(name, AmountEnum::Dynamic);
            }
        }

        // Check if the traveler names are valid
        {
            use crate::{
                chat::{ID as CHAT_ID, TABLE as CHAT_TB},
                traveler::{CHAT, NAME, TABLE as TRAVELER_TB},
            };
            const NAMES: &str = "names";

            let db = db().await;
            let select_res = db
                .query(format!(
                    "SELECT *
                        FROM {TRAVELER_TB}
                        WHERE
                            {CHAT} = ${CHAT_ID}
                            && {NAME} IN ${NAMES}",
                ))
                .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
                .bind((NAMES, split_among.keys().cloned().collect::<Vec<Name>>()))
                .await
                .and_then(|mut response| response.take::<Vec<Traveler>>(0));

            match select_res {
                Ok(travelers) => {
                    if travelers.len() == split_among.len() {
                        Ok((SplitAmongEnum::List, split_among))
                    } else {
                        let not_found = split_among
                            .keys()
                            .find(|name| !travelers.iter().any(|traveler| traveler.name == **name))
                            .unwrap(); // Can unwrap because there must be at least one travler that has not been found on db
                        return Err(AddExpenseError::TravelerNotFound {
                            name: not_found.to_owned(),
                        });
                    }
                }
                Err(err) => return Err(AddExpenseError::Generic(Box::new(err))),
            }
        }
    }
}

fn compute_shares(
    tot_amount: Decimal,
    mut split_among: BTreeMap<Name, AmountEnum>,
) -> Result<BTreeMap<Name, Decimal>, AddExpenseError> {
    let mut residual = tot_amount;
    let mut count_blanks = 0;

    for share in split_among.values() {
        match share {
            AmountEnum::Fixed(amount) => {
                residual -= amount;
                if residual.is_sign_negative() {
                    return Err(AddExpenseError::ExpenseTooHigh { tot_amount });
                }
            }
            AmountEnum::Dynamic => count_blanks += 1,
            AmountEnum::Percentage(_) => {} // Do nothing for now
        }
    }
    let residual_backup = residual;
    split_among.values_mut().for_each(|share| {
        // Evaluate percentages of the residual amount
        if let AmountEnum::Percentage(amount) = share {
            let fixed = residual_backup * *amount / Decimal::from(100);
            *share = AmountEnum::Fixed(fixed);
            residual -= fixed;
        }
    });

    if count_blanks == 0 && residual > Decimal::ZERO {
        return Err(AddExpenseError::ExpenseTooLow {
            expense: tot_amount - residual,
            tot_amount,
        });
    }

    let split_residual = residual.checked_div(Decimal::from(count_blanks));
    Ok(split_among
        .into_iter()
        .map(|(name, share)| {
            (
                name,
                match share {
                    AmountEnum::Fixed(amount) => amount,
                    AmountEnum::Dynamic => split_residual.unwrap(), // Can unwrap because count_blanks > 0
                    AmountEnum::Percentage(_) => unreachable!(), // Already converted to fixed amounts
                },
            )
        })
        .collect())
}

async fn relate_shares(
    paid_by: Traveler,
    expense: &Expense,
    shares: BTreeMap<Name, Decimal>,
) -> Result<(), surrealdb::Error> {
    use crate::{
        chat::TABLE as CHAT,
        expense::TABLE as EXPENSE,
        paid_for::TABLE as PAID_FOR_TB,
        split::{AMOUNT, TABLE as SPLIT_TB},
        traveler::{NAME, TABLE as TRAVELER_TB},
    };
    const PAID_BY: &str = "paid_by";

    let db = db().await;
    let mut query = db
        .query(BeginStatement::default())
        .query(format!("RELATE ${PAID_BY}->{PAID_FOR_TB}->${EXPENSE}"))
        .bind((PAID_BY, paid_by.id))
        .bind((EXPENSE, expense.id.clone()))
        .bind((CHAT, expense.chat.clone()));

    for (i, (name, amount)) in shares.into_iter().enumerate() {
        // Relate travelers with expense specifying their share of the expense
        query = query
            .query(format!(
                "RELATE (
                    SELECT * FROM {TRAVELER_TB} 
                    WHERE
                        {CHAT} = ${CHAT}
                        && {NAME} = ${NAME}_{i}
                )->{SPLIT_TB}->${EXPENSE}
                SET {AMOUNT} = <decimal> ${AMOUNT}_{i}"
            ))
            .bind((format!("{NAME}_{i}"), name))
            .bind((format!("{AMOUNT}_{i}"), amount));
    }

    query = query.query(CommitStatement::default());
    query.await.map(|_| {})
}
