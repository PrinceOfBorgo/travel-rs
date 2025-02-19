use crate::db::db;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct PaidFor {
    pub id: RecordId,
    pub r#in: RecordId,
    pub out: RecordId,
}

impl PaidFor {
    pub async fn db_relate(
        traveler: RecordId,
        expense: RecordId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!("RELATE ${IN}->{TABLE}->${OUT}",))
            .bind((IN, traveler))
            .bind((OUT, expense))
            .await
            .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
