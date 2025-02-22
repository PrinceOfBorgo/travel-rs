use crate::{db::db, traveler::Name};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use teloxide::types::ChatId;
use travel_rs_derive::Table;

const BALANCES_VIEW: &str = "v_balances";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Balance {
    pub id: RecordId,
    pub debt: Decimal,
    pub debtor_name: Name,
    pub creditor_name: Name,
    pub chat: RecordId,
}

impl Balance {
    pub async fn db_select(chat_id: ChatId) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {BALANCES_VIEW}
            WHERE {CHAT} = ${CHAT_ID}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }

    pub async fn db_select_by_name(
        chat_id: ChatId,
        name: Name,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::{
            chat::{ID as CHAT_ID, TABLE as CHAT_TB},
            traveler::NAME,
        };

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {BALANCES_VIEW}
            WHERE
                {CHAT} = ${CHAT_ID}
                && ({DEBTOR_NAME} = ${NAME} || {CREDITOR_NAME} = ${NAME})",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }
}
