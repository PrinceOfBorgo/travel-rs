use std::sync::Arc;

use crate::traveler::Name;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::any::Any};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

const FN_GET_BALANCES: &str = "fn::get_balances";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Balance {
    pub debt: Decimal,
    pub debtor_name: Name,
    pub creditor_name: Name,
    pub chat: RecordId,
}

impl Balance {
    pub async fn balances(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        db.query(format!(
            "SELECT *
            FROM {FN_GET_BALANCES}(${CHAT})
            ORDER BY 
                {DEBT} DESC, 
                {DEBTOR_NAME} ASC, 
                {CREDITOR_NAME} ASC",
        ))
        .bind((CHAT, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }

    pub async fn balances_by_name(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        name: Name,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::{chat::TABLE as CHAT_TB, traveler::NAME};

        db.query(format!(
            "SELECT *
            FROM {FN_GET_BALANCES}(${CHAT})
            WHERE {DEBTOR_NAME} = ${NAME} || {CREDITOR_NAME} = ${NAME}
            ORDER BY 
                {DEBT} DESC, 
                {DEBTOR_NAME} ASC, 
                {CREDITOR_NAME} ASC",
        ))
        .bind((CHAT, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }
}
