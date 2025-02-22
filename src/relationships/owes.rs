use crate::{db::db, traveler::Name};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use teloxide::types::ChatId;
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Owes {
    pub id: RecordId,
    pub amount: Decimal,
    pub r#in: RecordId,
    pub out: RecordId,
}

impl Owes {
    pub async fn db_relate(
        amount: Decimal,
        debitor: RecordId,
        creditor: RecordId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "RELATE ${IN}->{TABLE}->${OUT}
            SET {AMOUNT} = <decimal> ${AMOUNT}",
        ))
        .bind((IN, debitor))
        .bind((OUT, creditor))
        .bind((AMOUNT, amount))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }

    pub async fn db_select(chat_id: ChatId) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::{
            chat::{ID as CHAT_ID, TABLE as CHAT_TB},
            traveler::CHAT,
        };

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE {IN}.{CHAT}.{CHAT_ID} = ${CHAT_ID}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }
}
