use super::NameValidationError;
use crate::{
    i18n::{self, Translate, translate, translate_with_args},
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
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        use AddExpenseError::*;
        match self {
            RepeatedTravelerName { name } => translate_with_args(
                ctx,
                i18n::errors::ADD_EXPENSE_ERROR_REPEATED_TRAVELER_NAME,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            TravelerNotFound { name } => translate_with_args(
                ctx,
                i18n::errors::ADD_EXPENSE_ERROR_TRAVELER_NOT_FOUND,
                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
            ),
            ExpenseTooHigh { tot_amount } => translate_with_args(
                ctx,
                i18n::errors::ADD_EXPENSE_ERROR_EXPENSE_TOO_HIGH,
                &hashmap! {i18n::args::AMOUNT.into() => tot_amount.to_string().into()},
            ),
            ExpenseTooLow {
                expense,
                tot_amount,
            } => translate_with_args(
                ctx,
                i18n::errors::ADD_EXPENSE_ERROR_EXPENSE_TOO_LOW,
                &hashmap! {
                    i18n::args::EXPENSE.into() => expense.to_string().into(),
                    i18n::args::AMOUNT.into() => tot_amount.to_string().into()
                },
            ),
            InvalidFormat { input } => translate_with_args(
                ctx,
                i18n::errors::ADD_EXPENSE_ERROR_INVALID_FORMAT,
                &hashmap! {i18n::args::INPUT.into() => input.clone().into()},
            ),
            NoTravelersSpecified => {
                translate(ctx, i18n::errors::ADD_EXPENSE_ERROR_NO_TRAVELERS_SPECIFIED)
            }
            NameValidation(err) => err.translate(ctx),
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
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        use EndError::*;
        match self {
            ClosingDialogue => translate(ctx, i18n::errors::END_ERROR_CLOSING_DIALOGUE),
            NoExpenseCreated => translate(ctx, i18n::errors::END_ERROR_EXPENSE_CREATED),
            AddExpense(err) => err.translate(ctx),
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
