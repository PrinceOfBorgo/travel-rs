use std::sync::Arc;

use crate::db::Count;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::{
    RecordId, Surreal,
    engine::any::Any,
    sql::statements::{BeginStatement, CommitStatement},
};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct TransferredTo {
    pub id: RecordId,
    pub number: i64,
    pub amount: Decimal,
    pub r#in: RecordId,
    pub out: RecordId,
}

impl TransferredTo {
    pub async fn db_relate(
        db: Arc<Surreal<Any>>,
        amount: Decimal,
        from: RecordId,
        to: RecordId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::traveler::CHAT;

        db.query(BeginStatement::default())
            .query(format!(
                "LET $max = math::max(
                    SELECT VALUE {NUMBER} 
                    FROM {TABLE} 
                    WHERE {IN}.{CHAT} = ${IN}.{CHAT}
                ) ?? 0"
            ))
            .query(format!(
                "RELATE ${IN}->{TABLE}->${OUT}
                SET 
                    {AMOUNT} = <decimal> ${AMOUNT},
                    {NUMBER} = $max + 1",
            ))
            .query(CommitStatement::default())
            .bind((IN, from))
            .bind((OUT, to))
            .bind((AMOUNT, amount))
            .await
            .and_then(|mut response| response.take::<Option<Self>>(1))
    }

    pub async fn db_count(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        number: i64,
    ) -> Result<Option<Count>, surrealdb::Error> {
        use crate::{
            chat::{ID as CHAT_ID, TABLE as CHAT_TB},
            traveler::CHAT,
        };

        db.query(format!(
            "SELECT count()
            FROM {TABLE}
            WHERE 
                {IN}.{CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}
            GROUP BY count",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NUMBER, number))
        .await
        .and_then(|mut response| response.take::<Option<Count>>(0))
    }

    pub async fn db_delete(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        number: i64,
    ) -> Result<(), surrealdb::Error> {
        use crate::{
            chat::{ID as CHAT_ID, TABLE as CHAT_TB},
            traveler::CHAT,
        };

        db.query(format!(
            "DELETE {TABLE}
             WHERE 
                {IN}.{CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NUMBER, number))
        .await
        .map(|_| {})
    }
}
