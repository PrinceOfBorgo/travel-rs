use {
    rust_decimal::prelude::*,
    serde::{Deserialize, Serialize},
    surrealdb::RecordId,
    travel_rs_derive::Table,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Debt {
    pub debtor: RecordId,
    pub creditor: RecordId,
    pub debt: Decimal,
}
