use {crate::traveler::Name, rust_decimal::Decimal, std::fmt::Display};

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
    Generic(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for AddExpenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AddExpenseError::*;
        match self {
            RepeatedTravelerName { name } => write!(
                f,
                "Traveler `{name}` has already been added to the expense."
            ),
            TravelerNotFound { name } => write!(
                f,
                "Cannot find traveler `{name}` in the current travel plan."
            ),
            ExpenseTooHigh { tot_amount } => write!(
                f,
                "The expenses assigned to travelers exceed the total amount: {tot_amount}."
            ),
            ExpenseTooLow {
                expense,
                tot_amount,
            } => write!(
                f,
                "The expense ({expense}) is lower than the total amount: {tot_amount}."
            ),
            InvalidFormat { input } => write!(f, "Invalid format: `{input}`"),
            NoTravelersSpecified => write!(f, "No travelers have been specified."),
            Generic(err) => err.fmt(f),
        }
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

impl Display for EndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EndError::*;
        match self {
            ClosingDialogue => write!(f, "An error occured while closing the process."),
            NoExpenseCreated => write!(f, "No expense has been created."),
            AddExpense(err) => err.fmt(f),
            Generic(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for EndError {}
