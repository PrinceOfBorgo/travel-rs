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
        db.create((TABLE, id.0))
            .content((LAST_INTERACTION_UTC, Datetime(Utc::now())))
            .await
    }

    pub async fn db_update(id: ChatId) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.update((TABLE, id.0))
            .content((LAST_INTERACTION_UTC, Datetime(Utc::now())))
            .await
    }
}
