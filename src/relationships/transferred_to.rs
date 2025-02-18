use {
    crate::db::db,
    rust_decimal::prelude::*,
    serde::{Deserialize, Serialize},
    surrealdb::RecordId,
    travel_rs_derive::Table,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct TransferredTo {
    pub id: RecordId,
    pub amount: Decimal,
    pub r#in: RecordId,
    pub out: RecordId,
}

impl TransferredTo {
    pub async fn db_relate(
        amount: Decimal,
        from: RecordId,
        to: RecordId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let db = db().await;
        db.query(format!(
            "RELATE ${IN}->{TABLE}->${OUT}
            SET {AMOUNT} = <decimal> ${AMOUNT}",
        ))
        .bind((IN, from))
        .bind((OUT, to))
        .bind((AMOUNT, amount))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
