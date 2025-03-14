use crate::{
    consts::{INVALID_CHARS, RESERVED_KWORDS},
    db::{Count, db},
    errors::NameValidationError,
};

use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};
use surrealdb::RecordId;
use teloxide::types::ChatId;
use travel_rs_derive::Table;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Default for Name {
    fn default() -> Self {
        Self(String::from("Default"))
    }
}

impl FromStr for Name {
    type Err = NameValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if RESERVED_KWORDS.contains(&s.to_lowercase().as_str()) {
            Err(NameValidationError::ReservedKeyword(s.to_owned()))
        } else if let Some(c) = s.to_lowercase().chars().find(|c| INVALID_CHARS.contains(c)) {
            Err(NameValidationError::InvalidCharacter(s.to_owned(), c))
        } else if s.starts_with('/') {
            Err(NameValidationError::StartsWithSlash(s.to_owned()))
        } else {
            Ok(Name(s.to_owned()))
        }
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Name> for fluent::FluentValue<'_> {
    fn from(name: Name) -> Self {
        name.0.into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Traveler {
    pub id: RecordId,
    pub chat: RecordId,
    pub name: Name,
}

impl Traveler {
    pub async fn db_create(chat_id: ChatId, name: &Name) -> Result<Option<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "CREATE {TABLE}
            CONTENT {{
                {CHAT}: ${CHAT_ID}, 
                {NAME}: ${NAME},
            }}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name.clone()))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }

    pub async fn db_count(chat_id: ChatId, name: &Name) -> Result<Option<Count>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT count()
            FROM {TABLE}
            WHERE 
                {CHAT} = ${CHAT_ID}
                && {NAME} = ${NAME}
            GROUP BY count",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name.clone()))
        .await
        .and_then(|mut response| response.take::<Option<Count>>(0))
    }

    pub async fn db_delete(chat_id: ChatId, name: &Name) -> Result<(), surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "DELETE {TABLE}
            WHERE
                {CHAT} = ${CHAT_ID}
                && {NAME} = ${NAME}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name.clone()))
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
            ORDER BY {NAME} ASC",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }

    pub async fn db_select_by_name(
        chat_id: ChatId,
        name: Name,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use super::chat::{ID as CHAT_ID, TABLE as CHAT_TB};

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {TABLE}
            WHERE
                {CHAT} = ${CHAT_ID}
                && {NAME} = ${NAME}",
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
