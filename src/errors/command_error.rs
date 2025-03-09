use crate::{
    Context,
    i18n::{
        self, Translatable, translate, translate_default, translate_with_args,
        translate_with_args_default,
    },
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
    },
    SetLanguage {
        langid: LanguageIdentifier,
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
    ListExpenses,
    FindExpenses {
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
    ShowBalance {
        name: Name,
    },
    ShowBalances,
}

impl Translatable for CommandError {
    fn translate(&self, ctx: Arc<Mutex<Context>>) -> String {
        use CommandError::*;
        match self {
            EmptyInput => translate(ctx, i18n::errors::COMMAND_ERROR_EMPTY_INPUT),
            Help { command } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_HELP,
                &hashmap! {i18n::args::COMMAND.into() => command.into()},
            ),
            SetLanguage { langid } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_SET_LANGUAGE,
                &hashmap! {i18n::args::LANGID.into() => langid.to_string().into()},
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
            ListExpenses => translate(ctx, i18n::errors::COMMAND_ERROR_LIST_EXPENSES),
            FindExpenses { description } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_FIND_EXPENSES,
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
            ShowBalance { name } => translate_with_args(
                ctx,
                i18n::errors::COMMAND_ERROR_SHOW_BALANCE,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            ShowBalances => translate(ctx, i18n::errors::COMMAND_ERROR_SHOW_BALANCES),
        }
    }

    fn translate_default(&self) -> String {
        use CommandError::*;
        match self {
            EmptyInput => translate_default(i18n::errors::COMMAND_ERROR_EMPTY_INPUT),
            Help { command } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_HELP,
                &hashmap! {i18n::args::COMMAND.into() => command.into()},
            ),
            SetLanguage { langid } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_SET_LANGUAGE,
                &hashmap! {i18n::args::LANGID.into() => langid.to_string().into()},
            ),
            AddTraveler { name } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_ADD_TRAVELER,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            DeleteTraveler { name } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_DELETE_TRAVELER,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            ListTravelers => translate_default(i18n::errors::COMMAND_ERROR_LIST_TRAVELERS),
            DeleteExpense { number } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_DELETE_EXPENSE,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ),
            ListExpenses => translate_default(i18n::errors::COMMAND_ERROR_LIST_EXPENSES),
            FindExpenses { description } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_FIND_EXPENSES,
                &hashmap! {i18n::args::DESCRIPTION.into() => description.into()},
            ),
            ShowExpense { number } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_SHOW_EXPENSE,
                &hashmap! {i18n::args::NUMBER.into() => number.into()},
            ),
            Transfer {
                sender,
                receiver,
                amount,
            } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_TRANSFER,
                &hashmap! {
                    i18n::args::SENDER.into() => sender.clone().into(),
                    i18n::args::RECEIVER.into() => receiver.clone().into(),
                    i18n::args::AMOUNT.into() => amount.to_string().into()
                },
            ),
            ShowBalance { name } => translate_with_args_default(
                i18n::errors::COMMAND_ERROR_SHOW_BALANCE,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            ShowBalances => translate_default(i18n::errors::COMMAND_ERROR_SHOW_BALANCES),
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl std::error::Error for CommandError {}
