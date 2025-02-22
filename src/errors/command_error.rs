use crate::traveler::Name;
use rust_decimal::Decimal;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum CommandError {
    EmptyInput,
    Help {
        command: String,
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
    Transfer {
        from: Name,
        to: Name,
        amount: Decimal,
    },
    ShowBalance {
        name: Name,
    },
    ShowBalances,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CommandError::*;
        match self {
            EmptyInput => write!(f, "No input provided."),
            Help { command } => {
                write!(f, "No help available for command /{command}.")
            }
            AddTraveler { name } => write!(f, "Couldn't add traveler named \"{name}\"."),
            DeleteTraveler { name } => {
                write!(f, "Couldn't delete traveler named \"{name}\".")
            }
            ListTravelers => write!(f, "Couldn't list travelers."),
            DeleteExpense { number } => {
                write!(f, "Couldn't delete expense #{number}.")
            }
            ListExpenses => write!(f, "Couldn't list expenses."),
            FindExpenses { description } => write!(
                f,
                "Couldn't find expenses matching the specified description (~ \"{description}\")."
            ),
            Transfer { from, to, amount } => {
                write!(
                    f,
                    "Couldn't transfer {amount} from traveler \"{from}\" to \"{to}\"."
                )
            }
            ShowBalance { name } => {
                write!(f, "Couldn't show balance for traveler \"{name}\".")
            }
            ShowBalances => write!(f, "Couldn't show balances."),
        }
    }
}

impl std::error::Error for CommandError {}
