use super::AveragePerDay;
use crate::{
    expense::Expense,
    i18n::{self, Translate, TranslateWithArgs},
    money_wrapper::MoneyWrapper,
    utils::indent_multiline,
};
use maplit::hashmap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::{RecordId, Surreal, engine::any::Any};
use teloxide::types::ChatId;

const FN_EXPENSE_STATS: &str = "fn::expense_stats";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ExpenseStats {
    pub expenses_count: i64,
    pub sum: Decimal,
    pub mean: Decimal,
    pub min_expenses: Vec<Expense>,
    pub max_expenses: Vec<Expense>,
    pub average_per_day: Option<AveragePerDay>,
    pub oldest_expense: Option<Expense>,
    pub newest_expense: Option<Expense>,
}

impl ExpenseStats {
    pub async fn expense_stats(
        db: Arc<Surreal<Any>>,
        chat_id: ChatId,
    ) -> Result<Option<Self>, surrealdb::Error> {
        use crate::chat::TABLE as CHAT_TB;

        db.query(format!(
            "SELECT *
            FROM {FN_EXPENSE_STATS}($chat)",
        ))
        .bind(("chat", RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Option<Self>>(0))
    }
}

impl Translate for ExpenseStats {
    fn translate_with_indent(
        &self,
        ctx: Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        let sum = MoneyWrapper::new_with_context(self.sum, ctx.clone());
        let mean = MoneyWrapper::new_with_context(self.mean, ctx.clone());
        let min_expenses = indent_multiline(&self.min_expenses, ctx.clone(), indent_lvl);
        let max_expenses = indent_multiline(&self.max_expenses, ctx.clone(), indent_lvl);
        let average_per_day = self.average_per_day.as_ref().map_or(String::new(), |avg| {
            avg.translate_with_indent(ctx.clone(), indent_lvl)
        });
        let oldest_expense = self.oldest_expense.as_ref().map_or(String::new(), |e| {
            e.translate_with_indent(ctx.clone(), indent_lvl)
        });
        let newest_expense = self.newest_expense.as_ref().map_or(String::new(), |e| {
            e.translate_with_indent(ctx.clone(), indent_lvl)
        });

        i18n::format::FORMAT_EXPENSE_STATS.translate_with_args_indent(
            ctx,
            &hashmap! {
                i18n::args::COUNT.into() => self.expenses_count.into(),
                i18n::args::SUM.into() => sum.to_string().into(),
                i18n::args::MEAN.into() => mean.to_string().into(),
                i18n::args::MIN.into() => min_expenses.into(),
                i18n::args::MAX.into() => max_expenses.into(),
                i18n::args::AVERAGE_PER_DAY.into() => average_per_day.into(),
                i18n::args::OLDEST.into() => oldest_expense.into(),
                i18n::args::NEWEST.into() => newest_expense.into(),
            },
            indent_lvl,
        )
    }
}
