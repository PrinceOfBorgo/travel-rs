use crate::{
    db::{Count, db},
    i18n::{
        self, Translatable, translate_with_args, translate_with_args_default, types::FORMAT_EXPENSE,
    },
    money_wrapper::MoneyWrapper,
};
use maplit::hashmap;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use surrealdb::{
    RecordId,
    sql::statements::{BeginStatement, CommitStatement},
};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Expense {
    pub id: RecordId,
    pub chat: RecordId,
    pub number: i64,
    pub description: String,
    pub amount: Decimal,
}

impl Expense {
    pub async fn db_create(
        chat_id: ChatId,
        description: String,
        amount: Decimal,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(BeginStatement::default())
            .query(format!(
                "LET $max = math::max(
                    SELECT VALUE {NUMBER} 
                    FROM {TABLE} 
                    WHERE {CHAT} = ${CHAT_ID}
                ) ?? 0"
            ))
            .query(format!(
                "CREATE {TABLE}
                CONTENT {{
                    {CHAT}: ${CHAT_ID},
                    {DESCRIPTION}: ${DESCRIPTION},
                    {AMOUNT}: <decimal> ${AMOUNT},
                    {NUMBER}: $max + 1,
                }}",
            ))
            .query(CommitStatement::default())
            .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
            .bind((DESCRIPTION, description))
            .bind((AMOUNT, amount))
            .await
            .and_then(|mut response| response.take::<Option<Self>>(1))
    }

    pub async fn db_count(chat_id: ChatId, number: i64) -> Result<Option<Count>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT count()
            FROM {TABLE}
            WHERE 
                {CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}
            GROUP BY count",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NUMBER, number))
        .await
        .and_then(|mut response| response.take::<Option<Count>>(0))
    }

    pub async fn db_delete(chat_id: ChatId, number: i64) -> Result<(), surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "DELETE {TABLE}
             WHERE 
                {CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NUMBER, number))
        .await
        .map(|_| {})
    }

    pub async fn db_select(chat_id: ChatId) -> Result<Vec<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE {CHAT} = ${CHAT_ID}
            ORDER BY {NUMBER} ASC",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }

    pub async fn db_select_by_descr(
        chat_id: ChatId,
        fuzzy_descr: String,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};
        const FUZZY_DESCR: &str = "fuzzy_descr";

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE
                {CHAT} = ${CHAT_ID}
                && {DESCRIPTION} ~ ${FUZZY_DESCR}
            ORDER BY {NUMBER} ASC",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((FUZZY_DESCR, fuzzy_descr))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }
}

impl Translatable for Expense {
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        let amount = MoneyWrapper::new_with_context(self.amount, ctx.clone());
        translate_with_args(
            ctx,
            FORMAT_EXPENSE,
            &hashmap! {
                i18n::args::NUMBER.into() => self.number.into(),
                i18n::args::DESCRIPTION.into() => self.description.clone().into(),
                i18n::args::AMOUNT.into() => amount.to_string().into(),
            },
        )
    }

    fn translate_default(&self) -> String {
        translate_with_args_default(
            FORMAT_EXPENSE,
            &hashmap! {
                i18n::args::NUMBER.into() => self.number.into(),
                i18n::args::DESCRIPTION.into() => self.description.clone().into(),
                i18n::args::AMOUNT.into() => self.amount.to_string().into(),
            },
        )
    }
}

impl Display for Expense {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}
