use {
    crate::traveler::Traveler,
    rust_decimal::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Expense {
    pub chat_id: i64,
    pub expense_id: usize,
    pub description: String,
    pub amount: Decimal,
    pub payed_by: Box<Traveler>,
    /// key = traveler, value = (amount to pay, is percentage).
    pub split_among: HashMap<Box<Traveler>, (Decimal, bool)>,
}
