use crate::db::db;
use serde::{Deserialize, Serialize};
use surrealdb::{sql::Datetime, RecordId};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Table)]
pub struct Chat {
    id: RecordId,
    last_interaction_utc: Datetime,
}

impl Chat {
    pub async fn db_select_by_id(id: ChatId) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.select((TABLE, id.0)).await
    }

    pub async fn db_create(id: ChatId) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "CREATE ${ID}
            CONTENT {{
                {LAST_INTERACTION_UTC}: ${LAST_INTERACTION_UTC}, 
            }}",
        ))
        .bind((ID, RecordId::from_table_key(TABLE, id.0)))
        .bind((LAST_INTERACTION_UTC, Datetime::default()))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }

    pub async fn db_update(id: ChatId) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "UPDATE ${ID}
            CONTENT {{
                {LAST_INTERACTION_UTC}: ${LAST_INTERACTION_UTC}, 
            }}",
        ))
        .bind((ID, RecordId::from_table_key(TABLE, id.0)))
        .bind((LAST_INTERACTION_UTC, Datetime::default()))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
