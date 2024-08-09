use {
    crate::db::db,
    chrono::Utc,
    serde::{Deserialize, Serialize},
    surrealdb::sql::{Datetime, Thing},
    teloxide::types::ChatId,
    travel_rs_derive::Table,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Table)]
pub struct Chat {
    id: Thing,
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
        .bind((
            ID,
            Thing {
                tb: TABLE.to_owned(),
                id: id.0.into(),
            },
        ))
        .bind((LAST_INTERACTION_UTC, Datetime(Utc::now())))
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
        .bind((
            ID,
            Thing {
                tb: TABLE.to_owned(),
                id: id.0.into(),
            },
        ))
        .bind((LAST_INTERACTION_UTC, Datetime(Utc::now())))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
