use rust_decimal::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use surrealdb::{
    Surreal,
    engine::any::Any,
    sql::statements::{BeginStatement, CommitStatement},
};
use teloxide::types::ChatId;

use crate::errors::AddExpenseError;
use crate::tables::expense::Expense;
use crate::tables::traveler::{Name, Traveler};
use crate::utils::update_debts;

use super::ServiceError;

/// Enum representing the different ways to specify a share of an expense.
#[derive(Debug, Clone)]
pub enum AmountEnum {
    Fixed(Decimal),
    Percentage(Decimal),
    Dynamic,
}

/// Convert share definitions into final decimal amounts.
pub fn compute_shares(
    tot_amount: Decimal,
    mut split_among: BTreeMap<Name, AmountEnum>,
) -> Result<BTreeMap<Name, Decimal>, AddExpenseError> {
    // Start with the total amount to be split
    let mut residual = tot_amount;
    let mut count_dynamics = 0;

    // First pass: subtract fixed shares and count dynamic shares
    for share in split_among.values() {
        match share {
            AmountEnum::Fixed(amount) => {
                residual -= amount;
                // If the sum of fixed shares exceeds the total, return error
                if residual < Decimal::ZERO {
                    return Err(AddExpenseError::ExpenseTooHigh { tot_amount });
                }
            }
            AmountEnum::Dynamic => count_dynamics += 1,
            AmountEnum::Percentage(_) => {} // Percentages handled in next pass
        }
    }

    // Save the current residual for percentage calculation
    let residual_backup = residual;
    // Second pass: convert percentage shares to fixed amounts
    split_among.values_mut().for_each(|share| {
        if let AmountEnum::Percentage(amount) = share {
            // Calculate fixed amount for this percentage
            let fixed = residual_backup * *amount / Decimal::from(100);
            *share = AmountEnum::Fixed(fixed);
            residual -= fixed;
        }
    });

    // If there are no dynamic shares and residual remains, it's too low
    if count_dynamics == 0 && residual > Decimal::ZERO {
        return Err(AddExpenseError::ExpenseTooLow {
            expense: tot_amount - residual,
            tot_amount,
        });
    }

    // Divide the remaining residual equally among dynamic shares
    let split_residual = if count_dynamics > 0 {
        residual
            .checked_div(Decimal::from(count_dynamics))
            .expect("count_dynamics should be positive")
    } else {
        // No dynamic shares, so the remaining residual is not assigned to anyone
        Decimal::ZERO
    };

    // Build the final shares map
    Ok(split_among
        .into_iter()
        .map(|(name, share)| {
            let amount = match share {
                AmountEnum::Fixed(amount) => amount,
                AmountEnum::Dynamic => split_residual,
                AmountEnum::Percentage(_) => {
                    unreachable!("Already converted to fixed amounts")
                }
            };
            (name, amount)
        })
        .collect())
}

/// Create the PAID_FOR and SPLIT relationships for an expense inside a transaction.
pub async fn relate_shares(
    db: Arc<Surreal<Any>>,
    paid_by: &Traveler,
    expense: &Expense,
    shares: BTreeMap<Name, Decimal>,
) -> Result<(), surrealdb::Error> {
    use crate::{
        chat::TABLE as CHAT,
        expense::TABLE as EXPENSE,
        paid_for::TABLE as PAID_FOR_TB,
        split::{AMOUNT, TABLE as SPLIT_TB},
        traveler::{NAME, TABLE as TRAVELER_TB},
    };
    const PAID_BY: &str = "paid_by";

    let mut query = db
        .query(BeginStatement::default())
        .query(format!("RELATE ${PAID_BY}->{PAID_FOR_TB}->${EXPENSE}"))
        .bind((PAID_BY, paid_by.id.clone()))
        .bind((EXPENSE, expense.id.clone()))
        .bind((CHAT, expense.chat.clone()));

    for (i, (name, amount)) in shares.into_iter().enumerate() {
        // Relate travelers with expense specifying their share of the expense
        query = query
            .query(format!(
                "RELATE (
                    SELECT * FROM {TRAVELER_TB}
                    WHERE
                        {CHAT} = ${CHAT}
                        && {NAME} = ${NAME}_{i}
                )->{SPLIT_TB}->${EXPENSE}
                SET {AMOUNT} = <decimal> ${AMOUNT}_{i}"
            ))
            .bind((format!("{NAME}_{i}"), name))
            .bind((format!("{AMOUNT}_{i}"), amount));
    }

    query = query.query(CommitStatement::default());
    query.await.map(|_| {})
}

/// Create an expense with share relationships and update debts.
///
/// Returns the created `Expense` on success.
pub async fn create_expense(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    description: String,
    amount: Decimal,
    paid_by: &Traveler,
    shares: BTreeMap<Name, Decimal>,
) -> Result<Expense, ServiceError> {
    let expense = Expense::db_create(db.clone(), chat_id, description, amount)
        .await?
        .ok_or(ServiceError::NoExpenseCreated)?;

    if let Err(e) = relate_shares(db.clone(), paid_by, &expense, shares).await {
        let _ = Expense::db_delete_by_number(db, chat_id, expense.number).await;
        return Err(ServiceError::Database(e));
    }

    if let Err(e) = update_debts(db, chat_id).await {
        tracing::warn!("update_debts after add expense failed: {e}");
    }

    Ok(expense)
}

/// Delete an expense by number and update debts.
pub async fn delete_expense(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    number: i64,
) -> Result<(), ServiceError> {
    let count = Expense::db_count_by_number(db.clone(), chat_id, number).await?;
    if !count.map(|c| *c > 0).unwrap_or(false) {
        return Err(ServiceError::NotFound("Expense"));
    }

    Expense::db_delete_by_number(db.clone(), chat_id, number).await?;

    if let Err(e) = update_debts(db, chat_id).await {
        tracing::warn!("update_debts after delete expense failed: {e}");
    }

    Ok(())
}
