use crate::{
    Context,
    i18n::{self, Translate, TranslateWithArgs},
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
    ShowStats,
}

impl Translate for CommandError {
    fn translate_with_indent(&self, ctx: Arc<Mutex<Context>>, indent_lvl: usize) -> String {
        use CommandError::*;
        match self {
            EmptyInput => {
                i18n::errors::COMMAND_ERROR_EMPTY_INPUT.translate_with_indent(ctx, indent_lvl)
            }
            Help {
                command,
                best_match,
            } => {
                let err_msg1 = i18n::errors::COMMAND_ERROR_HELP.translate_with_args_indent(
                    ctx.clone(),
                    &hashmap! {i18n::args::COMMAND.into() => command.into()},
                    indent_lvl,
                );
                let err_msg2 = if let Some(best_match) = best_match {
                    i18n::commands::UNKNOWN_COMMAND_BEST_MATCH.translate_with_args_indent(
                        ctx,
                        &hashmap! {
                            i18n::args::COMMAND.into() => command.into(),
                            i18n::args::BEST_MATCH.into() => best_match.into()
                        },
                        indent_lvl,
                    )
                } else {
                    i18n::commands::UNKNOWN_COMMAND.translate_with_args_indent(
                        ctx,
                        &hashmap! {
                            i18n::args::COMMAND.into() => command.into(),
                        },
                        indent_lvl,
                    )
                };
                format!("{err_msg1}\n\n{err_msg2}")
            }
            SetLanguage { langid } => i18n::errors::COMMAND_ERROR_SET_LANGUAGE
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::LANGID.into() => langid.to_string().into()},
                    indent_lvl,
                ),
            SetCurrency { currency } => i18n::errors::COMMAND_ERROR_SET_CURRENCY
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::CURRENCY.into() => currency.into()},
                    indent_lvl,
                ),
            AddTraveler { name } => i18n::errors::COMMAND_ERROR_ADD_TRAVELER
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    indent_lvl,
                ),
            DeleteTraveler { name } => i18n::errors::COMMAND_ERROR_DELETE_TRAVELER
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    indent_lvl,
                ),
            ListTravelers => {
                i18n::errors::COMMAND_ERROR_LIST_TRAVELERS.translate_with_indent(ctx, indent_lvl)
            }
            DeleteExpense { number } => i18n::errors::COMMAND_ERROR_DELETE_EXPENSE
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    indent_lvl,
                ),
            ListExpenses { description } => i18n::errors::COMMAND_ERROR_LIST_EXPENSES
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::DESCRIPTION.into() => description.into()},
                    indent_lvl,
                ),
            ShowExpense { number } => i18n::errors::COMMAND_ERROR_SHOW_EXPENSE
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    indent_lvl,
                ),
            Transfer {
                sender,
                receiver,
                amount,
            } => i18n::errors::COMMAND_ERROR_TRANSFER.translate_with_args_indent(
                ctx,
                &hashmap! {
                    i18n::args::SENDER.into() => sender.clone().into(),
                    i18n::args::RECEIVER.into() => receiver.clone().into(),
                    i18n::args::AMOUNT.into() => amount.to_string().into()
                },
                indent_lvl,
            ),
            DeleteTransfer { number } => i18n::errors::COMMAND_ERROR_DELETE_TRANSFER
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NUMBER.into() => number.into()},
                    indent_lvl,
                ),
            ListTransfers { name } => i18n::errors::COMMAND_ERROR_LIST_TRANSFERS
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    indent_lvl,
                ),
            ShowBalances { name } => i18n::errors::COMMAND_ERROR_SHOW_BALANCES
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    indent_lvl,
                ),
            ShowStats => {
                i18n::errors::COMMAND_ERROR_SHOW_STATS.translate_with_indent(ctx, indent_lvl)
            }
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl std::error::Error for CommandError {}
