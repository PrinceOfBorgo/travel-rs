use {
    crate::db::{db, Count},
    rust_decimal::prelude::*,
    serde::{Deserialize, Serialize},
    std::fmt::{Display, Formatter},
    surrealdb::sql::{
        statements::{BeginStatement, CommitStatement},
        Thing,
    },
    teloxide::types::ChatId,
    travel_rs_derive::Table,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Table)]
pub struct Expense {
    pub id: Thing,
    pub chat: Thing,
    pub number: i64,
    pub description: String,
    pub amount: Decimal,
}

impl Expense {
    pub async fn db_create(
        chat_id: ChatId,
        description: &str,
        amount: Decimal,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(BeginStatement)
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
            .query(CommitStatement)
            .bind((
                CHAT_ID,
                Thing {
                    tb: CHAT_TB.to_owned(),
                    id: chat_id.0.into(),
                },
            ))
            .bind((DESCRIPTION, description))
            .bind((AMOUNT, amount))
            .await
            .and_then(|mut response| response.take::<Option<Self>>(1))
    }

    pub async fn db_count(chat_id: ChatId, number: i64) -> Result<Option<Count>, surrealdb::Error> {
        use crate::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT count()
            FROM {TABLE}
            WHERE 
                {CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}
            GROUP BY count",
        ))
        .bind((
            CHAT_ID,
            Thing {
                tb: CHAT_TB.to_owned(),
                id: chat_id.0.into(),
            },
        ))
        .bind((NUMBER, number))
        .await
        .and_then(|mut response| response.take::<Option<Count>>(0))
    }

    pub async fn db_delete(chat_id: ChatId, number: i64) -> Result<(), surrealdb::Error> {
        use crate::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "DELETE {TABLE}
             WHERE 
                {CHAT} = ${CHAT_ID}
                && {NUMBER} = ${NUMBER}",
        ))
        .bind((
            CHAT_ID,
            Thing {
                tb: CHAT_TB.to_owned(),
                id: chat_id.0.into(),
            },
        ))
        .bind((NUMBER, number))
        .await
        .map(|_| {})
    }

    pub async fn db_select<Out>(chat_id: ChatId) -> Result<Vec<Out>, surrealdb::Error>
    where
        Out: for<'de> Deserialize<'de>,
    {
        use crate::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE {CHAT} = ${CHAT_ID}",
        ))
        .bind((
            CHAT_ID,
            Thing {
                tb: CHAT_TB.to_owned(),
                id: chat_id.0.into(),
            },
        ))
        .await
        .and_then(|mut response| response.take::<Vec<Out>>(0))
    }

    pub async fn db_select_by_descr<Out>(
        chat_id: ChatId,
        fuzzy_descr: &str,
    ) -> Result<Vec<Out>, surrealdb::Error>
    where
        Out: for<'de> Deserialize<'de>,
    {
        use crate::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE
                {CHAT} = ${CHAT_ID}
                && {DESCRIPTION} ~ $fuzzy_descr",
        ))
        .bind((
            CHAT_ID,
            Thing {
                tb: CHAT_TB.to_owned(),
                id: chat_id.0.into(),
            },
        ))
        .bind(("fuzzy_descr", fuzzy_descr))
        .await
        .and_then(|mut response| response.take::<Vec<Out>>(0))
    }
}

impl Display for Expense {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let Expense {
            number,
            description,
            amount,
            ..
        } = self;

        write!(
            f,
            "Number: {number} - Description: {description}\nAmount: {amount}",
        )
    }
}
