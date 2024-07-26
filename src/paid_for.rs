use {
    crate::db::db,
    serde::{Deserialize, Serialize},
    surrealdb::sql::Thing,
    travel_rs_derive::Table,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Table)]
pub struct PaidFor {
    pub r#in: Thing,
    pub out: Thing,
}

impl PaidFor {
    pub async fn db_relate(
        traveler: Thing,
        expense: Thing,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!("RELATE ${IN}->{TABLE}->${OUT}",))
            .bind((IN, traveler))
            .bind((OUT, expense))
            .await
            .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
