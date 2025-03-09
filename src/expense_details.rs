use crate::{
    db::db,
    i18n::{
        self, Translatable, translate_with_args, translate_with_args_default,
        types::FORMAT_SHARE_DETAILS,
    },
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use surrealdb::RecordId;
use teloxide::types::ChatId;
use travel_rs_derive::Table;

const FN_GET_EXPENSE_DETAILS: &str = "fn::get_expense_details";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShareDetails {
    pub traveler_name: Name,
    pub amount: Decimal,
}

impl Translatable for ShareDetails {
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        translate_with_args(
            ctx,
            FORMAT_SHARE_DETAILS,
            &hashmap! {
                i18n::args::TRAVELER_NAME.into() => self.traveler_name.clone().into(),
                i18n::args::AMOUNT.into() => self.amount.to_string().into()
            },
        )
    }

    fn translate_default(&self) -> String {
        translate_with_args_default(
            FORMAT_SHARE_DETAILS,
            &hashmap! {
                i18n::args::TRAVELER_NAME.into() => self.traveler_name.clone().into(),
                i18n::args::AMOUNT.into() => self.amount.to_string().into()
            },
        )
    }
}

impl Display for ShareDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct ExpenseDetails {
    pub expense_number: i64,
    pub expense_description: String,
    pub expense_amount: Decimal,
    pub creditor_name: Name,
    pub shares: Vec<ShareDetails>,
    pub chat: RecordId,
}

impl ExpenseDetails {
    pub async fn expense_details(
        chat_id: ChatId,
        number: i64,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        let db = db().await;
        db.query(format!(
            "SELECT *
            FROM {FN_GET_EXPENSE_DETAILS}(${CHAT}, ${EXPENSE_NUMBER})",
        ))
        .bind((CHAT, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((EXPENSE_NUMBER, number))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}
