use crate::{
    i18n::{self, Translate, TranslateWithArgs},
    money_wrapper::MoneyWrapper,
    traveler::Name,
    utils::indent_multiline,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::{RecordId, Surreal, engine::any::Any};
use teloxide::types::ChatId;

const FN_TRAVELER_STATS: &str = "fn::traveler_stats";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TravelerStats {
    pub travelers_count: i64,
    pub travelers_paid_most: Vec<TravelerStatsAmount>,
    pub travelers_paid_least: Vec<TravelerStatsAmount>,
    pub travelers_pays_most_frequently: Vec<TravelerStatsFrequency>,
    pub travelers_pays_least_frequently: Vec<TravelerStatsFrequency>,
    pub major_creditors: Vec<TravelerStatsAmount>,
    pub major_debtors: Vec<TravelerStatsAmount>,
}

impl TravelerStats {
    pub async fn traveler_stats(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        db.query(format!(
            "SELECT *
            FROM {FN_TRAVELER_STATS}($chat)",
        ))
        .bind(("chat", RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}

impl Translate for TravelerStats {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        let travelers_count = self.travelers_count.to_string();
        let travelers_paid_most =
            indent_multiline(&self.travelers_paid_most, ctx.clone(), indent_lvl);
        let travelers_paid_least =
            indent_multiline(&self.travelers_paid_least, ctx.clone(), indent_lvl);
        let travelers_pays_most_frequently = indent_multiline(
            &self.travelers_pays_most_frequently,
            ctx.clone(),
            indent_lvl,
        );
        let travelers_pays_least_frequently = indent_multiline(
            &self.travelers_pays_least_frequently,
            ctx.clone(),
            indent_lvl,
        );
        let major_creditors = indent_multiline(&self.major_creditors, ctx.clone(), indent_lvl);
        let major_debtors = indent_multiline(&self.major_debtors, ctx.clone(), indent_lvl);

        i18n::format::FORMAT_TRAVELER_STATS.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::COUNT.into() => travelers_count.into(),
                i18n::args::TRAVELERS_PAID_MOST.into() => travelers_paid_most.into(),
                i18n::args::TRAVELERS_PAID_LEAST.into() => travelers_paid_least.into(),
                i18n::args::TRAVELERS_PAYS_MOST_FREQUENTLY.into() => travelers_pays_most_frequently.into(),
                i18n::args::TRAVELERS_PAYS_LEAST_FREQUENTLY.into() => travelers_pays_least_frequently.into(),
                i18n::args::MAJOR_DEBTORS.into() => major_debtors.into(),
                i18n::args::MAJOR_CREDITORS.into() => major_creditors.into(),
            },
            indent_lvl,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TravelerStatsAmount {
    pub traveler_name: Name,
    pub amount: Decimal,
}

impl Translate for TravelerStatsAmount {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        let total_paid = MoneyWrapper::new_with_context(self.amount, ctx.clone());
        i18n::format::FORMAT_TRAVELER_STATS_AMOUNT.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::TRAVELER_NAME.into() => self.traveler_name.clone().into(),
                i18n::args::AMOUNT.into() => total_paid.to_string().into(),
            },
            indent_lvl,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TravelerStatsFrequency {
    pub traveler_name: Name,
    pub count: i64,
}

impl Translate for TravelerStatsFrequency {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        i18n::format::FORMAT_TRAVELER_STATS_FREQUENCY.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::TRAVELER_NAME.into() => self.traveler_name.clone().into(),
                i18n::args::COUNT.into() => self.count.into(),
            },
            indent_lvl,
        )
    }
}
