use crate::{db::db, traveler::Name};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use teloxide::types::ChatId;
use travel_rs_derive::Table;

const FN_GET_EXPENSE_DETAILS: &str = "fn::get_expense_details";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShareDetails {
    pub traveler_name: Name,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct ExpenseDetails {
    pub expense_number: i64,
    pub expense_description: String,
    pub expense_amount: Decimal,
    pub creditor_name: Name,
    pub shares: Vec<ShareDetails>,
    pub chat: RecordId,
}

impl ExpenseDetails {
    pub async fn expense_details(
        chat_id: ChatId,
        number: i64,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {FN_GET_EXPENSE_DETAILS}(${CHAT}, ${EXPENSE_NUMBER})",
        ))
        .bind((CHAT, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((EXPENSE_NUMBER, number))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
