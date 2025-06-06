use crate::{
    Context,
    i18n::{self, Translate, translate, translate_with_args},
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};
use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone)]
pub enum CommandError {
    EmptyInput,
    Help {
        command: String,
        best_match: Option<String>,
    },
    SetLanguage {
        langid: LanguageIdentifier,
    },
    SetCurrency {
        currency: String,
    },
    AddTraveler {
        name: Name,
    },
    DeleteTraveler {
        name: Name,
    },
    ListTravelers,
    DeleteExpense {
        number: i64,
    },
    ListExpenses {
        description: String,
    },
    ShowExpense {
        number: i64,
    },
    Transfer {
        sender: Name,
        receiver: Name,
        amount: Decimal,
    },
    DeleteTransfer {
        number: i64,
    },
    ListTransfers {
        name: Name,
    },
    ShowBalances {
        name: Name,
    },
}

impl Translate for CommandError {
    fn translate(&self, ctx: Arc<Mutex<Context>>) -> String {
        use CommandError::*;
        match self {
            EmptyInput => translate(ctx, i18n::errors::COMMAND_ERROR_EMPTY_INPUT),
            Help {
                command,
                best_match,
            } => {
                let err_msg1 = translate_with_args(
                    ctx.clone(),
                    i18n::errors::COMMAND_ERROR_HELP,
                    &hashmap! {i18n::args::COMMAND.into() => command.into()},
                );
                let err_msg2 = if let Some(best_match) = best_match {
                    translate_with_args(
                        ctx,
                        i18n::commands::UNKNOWN_COMMAND_BEST_MATCH,
                        &hashmap! {
                            i18n::args::COMMAND.into() => command.into(),
                            i18n::args::BEST_MATCH.into() => best_match.into()
                        },
                    )
                } else {
                    translate_with_args(
                        ctx,
                        i18n::commands::UNKNOWN_COMMAND,
                        &hashmap! {
                            i18n::args::COMMAND.into() => command.into(),
                        },
                    )
                };
                format!("{err_msg1}\n\n{err_msg2}")
            }
            SetLanguage { langid } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_SET_LANGUAGE,
                &hashmap! {i18n::args::LANGID.into() => langid.to_string().into()},
            ),
            SetCurrency { currency } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_SET_CURRENCY,
                &hashmap! {i18n::args::CURRENCY.into() => currency.into()},
            ),
            AddTraveler { name } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_ADD_TRAVELER,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            DeleteTraveler { name } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_DELETE_TRAVELER,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            ListTravelers => translate(ctx, i18n::errors::COMMAND_ERROR_LIST_TRAVELERS),
            DeleteExpense { number } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_DELETE_EXPENSE,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ),
            ListExpenses { description } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_LIST_EXPENSES,
                &hashmap! {i18n::args::DESCRIPTION.into() => description.into()},
            ),
            ShowExpense { number } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_SHOW_EXPENSE,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ),
            Transfer {
                sender,
                receiver,
                amount,
            } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_TRANSFER,
                &hashmap! {
                    i18n::args::SENDER.into() => sender.clone().into(),
                    i18n::args::RECEIVER.into() => receiver.clone().into(),
                    i18n::args::AMOUNT.into() => amount.to_string().into()
                },
            ),
            DeleteTransfer { number } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_DELETE_TRANSFER,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ),
            ListTransfers { name } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_LIST_TRANSFERS,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            ShowBalances { name } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_SHOW_BALANCES,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl std::error::Error for CommandError {}
