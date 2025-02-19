use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Debt {
    pub debtor: RecordId,
    pub creditor: RecordId,
    pub debt: Decimal,
}
