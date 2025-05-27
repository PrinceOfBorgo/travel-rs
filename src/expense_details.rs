use crate::{
    i18n::{self, Translate, format::FORMAT_SHARE_DETAILS, translate_with_args},
    money_wrapper::MoneyWrapper,
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};
use surrealdb::{RecordId, Surreal, engine::any::Any};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

const FN_GET_EXPENSE_DETAILS: &str = "fn::get_expense_details";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShareDetails {
    pub traveler_name: Name,
    pub amount: Decimal,
}

impl Translate for ShareDetails {
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        let amount = MoneyWrapper::new_with_context(self.amount, ctx.clone());
        translate_with_args(
            ctx,
            FORMAT_SHARE_DETAILS,
            &hashmap! {
                i18n::args::TRAVELER_NAME.into() => self.traveler_name.clone().into(),
                i18n::args::AMOUNT.into() => amount.to_string().into()
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
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        number: i64,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

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

impl Translate for ExpenseDetails {
    fn translate(&self, ctx: Arc<std::sync::Mutex<crate::Context>>) -> String {
        let amount = MoneyWrapper::new_with_context(self.expense_amount, ctx.clone());
        let shares_str = self
            .shares
            .iter()
            .map(|share_details| share_details.translate(ctx.clone()))
            .collect::<Vec<_>>()
            .join("\n");
        translate_with_args(
            ctx,
            i18n::format::FORMAT_EXPENSE_DETAILS,
            &hashmap! {
                i18n::args::NUMBER.into() => self.expense_number.to_string().into(),
                i18n::args::DESCRIPTION.into() => self.expense_description.clone().into(),
                i18n::args::AMOUNT.into() => amount.to_string().into(),
                i18n::args::CREDITOR.into() => self.creditor_name.clone().into(),
                i18n::args::SHARES.into() => shares_str.into(),
            },
        )
    }
}
