use std::fmt::Display;

use crate::traveler::Name;

#[derive(Debug)]
pub enum Error {
    EmptyInput,
    AddTraveler { name: Name },
    DeleteTraveler { name: Name },
    ListTravelers,
    AddExpense,
    DeleteExpense { expense_id: usize },
    ListExpenses,
}
impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            EmptyInput => write!(f, "No input provided."),
            AddTraveler { name } => write!(f, "Couldn't add traveler named \"{name}\"."),
            DeleteTraveler { name } => {
                write!(f, "Couldn't delete traveler named \"{name}\".")
            }
            ListTravelers => write!(f, "Couldn't list travelers."),
            AddExpense => write!(f, "Couldn't add expense."),
            DeleteExpense { expense_id } => {
                write!(f, "Couldn't delete expense with ID \"{expense_id}\".")
            }
            ListExpenses => write!(f, "Couldn't list expenses."),
        }
    }
}
