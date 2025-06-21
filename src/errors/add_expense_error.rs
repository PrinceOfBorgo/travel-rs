use super::NameValidationError;
use crate::{
    i18n::{self, Translate, TranslateWithArgs},
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use std::fmt::Display;

#[derive(Debug)]
pub enum AddExpenseError {
    RepeatedTravelerName {
        name: Name,
    },
    TravelerNotFound {
        name: Name,
    },
    ExpenseTooHigh {
        tot_amount: Decimal,
    },
    ExpenseTooLow {
        expense: Decimal,
        tot_amount: Decimal,
    },
    InvalidFormat {
        input: String,
    },
    NoTravelersSpecified,
    NameValidation(NameValidationError),
    Generic(Box<dyn std::error::Error + Send + Sync>),
}

impl Translate for AddExpenseError {
    fn translate_with_indent(
        &self,
        ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        use AddExpenseError::*;
        match self {
            RepeatedTravelerName { name } => i18n::errors::ADD_EXPENSE_ERROR_REPEATED_TRAVELER_NAME
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    indent_lvl,
                ),
            TravelerNotFound { name } => i18n::errors::ADD_EXPENSE_ERROR_TRAVELER_NOT_FOUND
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                    indent_lvl,
                ),
            ExpenseTooHigh { tot_amount } => i18n::errors::ADD_EXPENSE_ERROR_EXPENSE_TOO_HIGH
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::AMOUNT.into() => tot_amount.to_string().into()},
                    indent_lvl,
                ),
            ExpenseTooLow {
                expense,
                tot_amount,
            } => i18n::errors::ADD_EXPENSE_ERROR_EXPENSE_TOO_LOW.translate_with_args_indent(
                ctx,
                &hashmap! {
                    i18n::args::EXPENSE.into() => expense.to_string().into(),
                    i18n::args::AMOUNT.into() => tot_amount.to_string().into()
                },
                indent_lvl,
            ),
            InvalidFormat { input } => i18n::errors::ADD_EXPENSE_ERROR_INVALID_FORMAT
                .translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::INPUT.into() => input.clone().into()},
                    indent_lvl,
                ),
            NoTravelersSpecified => i18n::errors::ADD_EXPENSE_ERROR_NO_TRAVELERS_SPECIFIED
                .translate_with_indent(ctx, indent_lvl),
            NameValidation(err) => err.translate_with_indent(ctx, indent_lvl),
            Generic(err) => err.to_string(),
        }
    }
}

impl Display for AddExpenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl std::error::Error for AddExpenseError {}

#[derive(Debug)]
pub enum EndError {
    ClosingDialogue,
    NoExpenseCreated,
    AddExpense(AddExpenseError),
    Generic(Box<dyn std::error::Error + Send + Sync>),
}

impl Translate for EndError {
    fn translate_with_indent(
        &self,
        ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        use EndError::*;
        match self {
            ClosingDialogue => {
                i18n::errors::END_ERROR_CLOSING_DIALOGUE.translate_with_indent(ctx, indent_lvl)
            }
            NoExpenseCreated => {
                i18n::errors::END_ERROR_EXPENSE_CREATED.translate_with_indent(ctx, indent_lvl)
            }
            AddExpense(err) => err.translate_with_indent(ctx, indent_lvl),
            Generic(err) => err.to_string(),
        }
    }
}

impl Display for EndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl std::error::Error for EndError {}
