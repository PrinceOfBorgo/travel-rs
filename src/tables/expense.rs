use crate::{
    db::Count,
    i18n::{self, ToFluentDateTime, Translate, format::FORMAT_EXPENSE, translate_with_args},
    money_wrapper::MoneyWrapper,
};
use maplit::hashmap;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};
use surrealdb::{
    Datetime, RecordId, Surreal,
    engine::any::Any,
    sql::statements::{BeginStatement, CommitStatement},
};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

use super::traveler::Traveler;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Expense {
    pub id: RecordId,
    pub chat: RecordId,
    pub number: i64,
    pub description: String,
    pub amount: Decimal,
    pub timestamp_utc: Datetime,
}

impl Expense {
    pub async fn db_create(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        description: String,
        amount: Decimal,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

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

    pub async fn db_count_by_number(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        number: i64,
    ) -> Result<Option<Count>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

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

    pub async fn db_delete_by_number(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        number: i64,
    ) -> Result<(), surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

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

    pub async fn db_select(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

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
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        fuzzy_descr: String,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};
        const FUZZY_DESCR: &str = "fuzzy_descr";

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

    pub async fn db_select_by_payer(
        db: Arc<Surreal<Any>>,
        traveler: Traveler,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::{paid_for::TABLE as PAID_FOR, traveler::TABLE as TRAVELER};

        db.query(format!("${TRAVELER}->{PAID_FOR}->{TABLE}.*"))
            .bind((TRAVELER, traveler))
            .await
            .and_then(|mut response| response.take::<Vec<Self>>(0))
    }

    pub async fn db_select_by_number(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        number: i64,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE 
                {CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NUMBER, number))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}

impl Translate for Expense {
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        let amount = MoneyWrapper::new_with_context(self.amount, ctx.clone());
        translate_with_args(
            ctx,
            FORMAT_EXPENSE,
            &hashmap! {
                i18n::args::NUMBER.into() => self.number.into(),
                i18n::args::DESCRIPTION.into() => self.description.clone().into(),
                i18n::args::AMOUNT.into() => amount.to_string().into(),
                i18n::args::DATETIME.into() => self.timestamp_utc.to_fluent_datetime().unwrap().into(),
            },
        )
    }
}

impl Display for Expense {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}
