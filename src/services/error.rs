use std::fmt;

use crate::errors::AddExpenseError;
use crate::tables::expense::Expense;

/// Errors returned by the service layer.
///
/// These are framework-agnostic: callers (Telegram commands, HTTP handlers) map them to their own response types.
#[derive(Debug)]
pub enum ServiceError {
    AlreadyExists(&'static str),
    NotFound(&'static str),
    HasAssociatedExpenses(Vec<Expense>),
    EmptyInput(&'static str),
    ShareComputation(AddExpenseError),
    NoExpenseCreated,
    LanguageNotAvailable {
        requested: String,
        available: Vec<String>,
    },
    Database(surrealdb::Error),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists(what) => write!(f, "{what} already exists"),
            Self::NotFound(what) => write!(f, "{what} not found"),
            Self::HasAssociatedExpenses(_) => {
                write!(f, "Traveler has associated expenses and cannot be deleted")
            }
            Self::EmptyInput(what) => write!(f, "{what} must not be empty"),
            Self::ShareComputation(e) => write!(f, "Share computation error: {e}"),
            Self::NoExpenseCreated => write!(f, "Failed to create expense"),
            Self::LanguageNotAvailable {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Language '{requested}' not available. Available: {}",
                    available.join(", ")
                )
            }
            Self::Database(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<surrealdb::Error> for ServiceError {
    fn from(e: surrealdb::Error) -> Self {
        Self::Database(e)
    }
}

impl From<AddExpenseError> for ServiceError {
    fn from(e: AddExpenseError) -> Self {
        Self::ShareComputation(e)
    }
}
