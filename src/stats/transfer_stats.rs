use super::AveragePerDay;
use crate::{
    i18n::{self, Translate, TranslateWithArgs},
    money_wrapper::MoneyWrapper,
    transfer::Transfer,
    utils::indent_multiline,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::{RecordId, Surreal, engine::any::Any};
use teloxide::types::ChatId;

const FN_TRANSFER_STATS: &str = "fn::transfer_stats";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TransferStats {
    pub transfers_count: i64,
    pub sum: Decimal,
    pub mean: Decimal,
    pub min_transfers: Vec<Transfer>,
    pub max_transfers: Vec<Transfer>,
    pub average_per_day: Option<AveragePerDay>,
    pub oldest_transfer: Option<Transfer>,
    pub newest_transfer: Option<Transfer>,
}

impl TransferStats {
    pub async fn transfer_stats(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        db.query(format!(
            "SELECT *
            FROM {FN_TRANSFER_STATS}($chat)",
        ))
        .bind(("chat", RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}

impl Translate for TransferStats {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        let transfers_count = self.transfers_count.to_string();
        let sum = MoneyWrapper::new_with_context(self.sum, ctx.clone());
        let mean = MoneyWrapper::new_with_context(self.mean, ctx.clone());
        let min_transfers = indent_multiline(&self.min_transfers, ctx.clone(), indent_lvl);
        let max_transfers = indent_multiline(&self.max_transfers, ctx.clone(), indent_lvl);
        let average_per_day = self.average_per_day.as_ref().map_or(String::new(), |avg| {
            avg.translate_with_indent(ctx.clone(), indent_lvl)
        });
        let oldest_transfer = self.oldest_transfer.as_ref().map_or(String::new(), |t| {
            t.translate_with_indent(ctx.clone(), indent_lvl)
        });
        let newest_transfer = self.newest_transfer.as_ref().map_or(String::new(), |t| {
            t.translate_with_indent(ctx.clone(), indent_lvl)
        });

        i18n::format::FORMAT_TRANSFER_STATS.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::COUNT.into() => transfers_count.into(),
                i18n::args::SUM.into() => sum.to_string().into(),
                i18n::args::MEAN.into() => mean.to_string().into(),
                i18n::args::MIN.into() => min_transfers.into(),
                i18n::args::MAX.into() => max_transfers.into(),
                i18n::args::AVERAGE_PER_DAY.into() => average_per_day.into(),
                i18n::args::OLDEST.into() => oldest_transfer.into(),
                i18n::args::NEWEST.into() => newest_transfer.into(),
            },
            indent_lvl,
        )
    }
}
