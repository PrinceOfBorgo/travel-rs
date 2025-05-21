use std::sync::Arc;

use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::any::Any};
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct PaidFor {
    pub id: RecordId,
    pub r#in: RecordId,
    pub out: RecordId,
}

impl PaidFor {
    pub async fn db_relate(
        db: Arc<Surreal<Any>>,
        traveler: RecordId,
        expense: RecordId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        db.query(format!("RELATE ${IN}->{TABLE}->${OUT}",))
            .bind((IN, traveler))
            .bind((OUT, expense))
            .await
            .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
