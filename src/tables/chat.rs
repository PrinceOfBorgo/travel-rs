use crate::db::db;
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, sql::Datetime};
use teloxide::types::ChatId;
use travel_rs_derive::Table;
use unic_langid::LanguageIdentifier;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Table)]
pub struct Chat {
    pub id: RecordId,
    pub last_interaction_utc: Datetime,
    pub lang: String,
}

impl Chat {
    pub async fn db_select_by_id(id: ChatId) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.select((TABLE, id.0)).await
    }

    pub async fn db_create(id: ChatId, lang: String) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "CREATE ${ID}
            CONTENT {{
                {LAST_INTERACTION_UTC}: ${LAST_INTERACTION_UTC}, 
                {LANG}: ${LANG},
            }}",
        ))
        .bind((ID, RecordId::from_table_key(TABLE, id.0)))
        .bind((LAST_INTERACTION_UTC, Datetime::default()))
        .bind((LANG, lang))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }

    pub async fn db_update_last_interaction_utc(
        id: ChatId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "UPDATE ${ID}
            SET {LAST_INTERACTION_UTC} = ${LAST_INTERACTION_UTC}",
        ))
        .bind((ID, RecordId::from_table_key(TABLE, id.0)))
        .bind((LAST_INTERACTION_UTC, Datetime::default()))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }

    pub async fn db_update_lang(
        id: ChatId,
        langid: &LanguageIdentifier,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "UPDATE ${ID}
            SET {LANG} = ${LANG}",
        ))
        .bind((ID, RecordId::from_table_key(TABLE, id.0)))
        .bind((LANG, langid.to_string()))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
