use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use crate::{
    i18n::{self, Translate, format::FORMAT_TRANSFER, translate_with_args},
    money_wrapper::MoneyWrapper,
    traveler::Name,
};
use maplit::hashmap;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::any::Any};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

const FN_GET_TRANSFERS: &str = "fn::get_transfers";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Transfer {
    pub number: i64,
    pub amount: Decimal,
    pub sender_name: Name,
    pub receiver_name: Name,
    pub chat: RecordId,
}

impl Transfer {
    pub async fn transfers(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        db.query(format!(
            "SELECT *
            FROM {FN_GET_TRANSFERS}(${CHAT})
            ORDER BY {NUMBER} ASC",
        ))
        .bind((CHAT, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }

    pub async fn transfers_by_name(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
        name: Name,
    ) -> Result<Vec<Self>, surrealdb::Error> {
        use crate::{
            chat::TABLE as CHAT_TB,
            traveler::{CHAT, NAME},
        };

        db.query(format!(
            "SELECT *
            FROM {FN_GET_TRANSFERS}(${CHAT})
            WHERE {SENDER_NAME} = ${NAME} || {RECEIVER_NAME} = ${NAME}
            ORDER BY {NUMBER} ASC",
        ))
        .bind((CHAT, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .bind((NAME, name))
        .await
        .and_then(|mut response| response.take::<Vec<Self>>(0))
    }
}

impl Translate for Transfer {
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        let amount = MoneyWrapper::new_with_context(self.amount, ctx.clone());
        translate_with_args(
            ctx,
            FORMAT_TRANSFER,
            &hashmap! {
                i18n::args::NUMBER.into() => self.number.into(),
                i18n::args::SENDER.into() => self.sender_name.clone().into(),
                i18n::args::RECEIVER.into() => self.receiver_name.clone().into(),
                i18n::args::AMOUNT.into() => amount.to_string().into(),
            },
        )
    }
}

impl Display for Transfer {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}
