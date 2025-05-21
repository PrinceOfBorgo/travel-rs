use std::sync::Arc;

use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::any::Any};
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Split {
    pub id: RecordId,
    pub amount: Decimal,
    pub r#in: RecordId,
    pub out: RecordId,
}

impl Split {
    pub async fn db_relate(
        db: Arc<Surreal<Any>>,
        amount: Decimal,
        traveler: RecordId,
        expense: RecordId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        db.query(format!(
            "RELATE ${IN}->{TABLE}->${OUT}
            SET {AMOUNT} = <decimal> ${AMOUNT}",
        ))
        .bind((IN, traveler))
        .bind((OUT, expense))
        .bind((AMOUNT, amount))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
