mod expense_stats;
mod transfer_stats;
mod traveler_stats;

pub use expense_stats::ExpenseStats;
pub use transfer_stats::TransferStats;
pub use traveler_stats::TravelerStats;

use crate::{
    i18n::{self, ToFluentDateTime, Translate, TranslateWithArgs},
    money_wrapper::MoneyWrapper,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::Datetime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Stats {
    pub expense_stats: ExpenseStats,
    pub transfer_stats: TransferStats,
    pub traveler_stats: TravelerStats,
}

impl Stats {
    pub async fn stats(
        db: Arc<surrealdb::Surreal<surrealdb::engine::any::Any>>,
        chat_id: teloxide::types::ChatId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        let Some(expense_stats) = ExpenseStats::expense_stats(db.clone(), chat_id).await? else {
            return Ok(None);
        };
        let Some(transfer_stats) = TransferStats::transfer_stats(db.clone(), chat_id).await? else {
            return Ok(None);
        };
        let Some(traveler_stats) = TravelerStats::traveler_stats(db.clone(), chat_id).await? else {
            return Ok(None);
        };

        Ok(Some(Self {
            expense_stats,
            transfer_stats,
            traveler_stats,
        }))
    }
}

impl Translate for Stats {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        let expense_stats = self.expense_stats.translate(ctx.clone());
        let transfer_stats = self.transfer_stats.translate(ctx.clone());
        let traveler_stats = self.traveler_stats.translate(ctx.clone());

        i18n::format::FORMAT_STATS.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::EXPENSE_STATS.into() => expense_stats.into(),
                i18n::args::TRANSFER_STATS.into() => transfer_stats.into(),
                i18n::args::TRAVELER_STATS.into() => traveler_stats.into(),
            },
            indent_lvl,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AveragePerDay {
    pub amount: Decimal,
    pub oldest_timestamp: Datetime,
    pub now: Datetime,
}

impl Translate for AveragePerDay {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        let amount = MoneyWrapper::new_with_context(self.amount, ctx.clone());
        i18n::format::FORMAT_AVERAGE_PER_DAY.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::AMOUNT.into() => amount.to_string().into(),
                i18n::args::OLDEST_TIMESTAMP.into() => self.oldest_timestamp.to_fluent_datetime().unwrap().into(),
                i18n::args::NOW.into() => self.now.to_fluent_datetime().unwrap().into(),
            },
            indent_lvl,
        )
    }
}
