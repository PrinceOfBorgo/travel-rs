use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use surrealdb::{
    RecordId, Surreal,
    engine::any::Any,
    sql::statements::{BeginStatement, CommitStatement},
};
use teloxide::types::ChatId;
use travel_rs_derive::Table;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Table)]
pub struct Debt {
    pub debtor: RecordId,
    pub creditor: RecordId,
    pub debt: Decimal,
}

const FN_CALC_DEBTS: &str = "fn::calc_debts";
const FN_CLEAR_DEBTS: &str = "fn::clear_debts";

/// Simplifies a list of debts by calculating the net balance for each participant
/// and creating new debt transactions that reflect the simplified balances.
fn simplify_balances(debts: &mut Vec<Debt>) {
    let keys: HashMap<String, RecordId> = debts
        .iter()
        .flat_map(|d| {
            vec![
                (d.debtor.to_string(), d.debtor.clone()),
                (d.creditor.to_string(), d.creditor.clone()),
            ]
        })
        .collect();

    let mut balance_map: HashMap<String, Decimal> = HashMap::new();

    for debt in debts.iter() {
        *balance_map
            .entry(debt.debtor.to_string())
            .or_insert(Decimal::ZERO) -= debt.debt;
        *balance_map
            .entry(debt.creditor.to_string())
            .or_insert(Decimal::ZERO) += debt.debt;
    }

    let mut creditors: Vec<_> = balance_map
        .clone()
        .into_iter()
        .filter(|&(_, v)| v > Decimal::ZERO)
        .collect();
    let mut debtors: Vec<_> = balance_map
        .into_iter()
        .filter(|&(_, v)| v < Decimal::ZERO)
        .collect();

    creditors.sort_by_key(|a| std::cmp::Reverse(a.1));
    debtors.sort_by_key(|a| a.1);

    debts.clear();

    while !debtors.is_empty() && !creditors.is_empty() {
        let (debtor, mut debtor_amount) = debtors.pop().unwrap();
        let (creditor, mut creditor_amount) = creditors.pop().unwrap();

        let amount = debtor_amount.abs().min(creditor_amount);

        let debtor_id = keys
            .get(&debtor)
            .unwrap_or_else(|| panic!("Debtor with id {debtor} should exist"))
            .clone();
        let creditor_id = keys
            .get(&creditor)
            .unwrap_or_else(|| panic!("Creditor with id {creditor} should exist"))
            .clone();

        debts.push(Debt {
            debtor: debtor_id,
            creditor: creditor_id,
            debt: amount,
        });

        debtor_amount += amount;
        if debtor_amount < Decimal::ZERO {
            debtors.push((debtor, debtor_amount));
        }

        creditor_amount -= amount;
        if creditor_amount > Decimal::ZERO {
            creditors.push((creditor, creditor_amount));
        }
    }
}

/// Updates the debts for a given chat by recalculating the net balances and simplifying the transactions.
/// This function retrieves the current debts from the database, simplifies them, and then updates the database with the new simplified debts.
pub async fn update_debts(db: Arc<Surreal<Any>>, chat_id: ChatId) -> Result<(), surrealdb::Error> {
    use crate::{
        chat::{ID as CHAT_ID, TABLE as CHAT_TB},
        owes::{AMOUNT, TABLE as OWES},
    };

    let mut debts = db
        .query(format!(
            "SELECT {DEBTOR}, {CREDITOR}, {DEBT} 
            FROM {FN_CALC_DEBTS}(${CHAT_ID})"
        ))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)))
        .await
        .and_then(|mut response| response.take::<Vec<Debt>>(0))?;

    let mut query = db
        .query(BeginStatement::default())
        .query(format!("{FN_CLEAR_DEBTS}(${CHAT_ID})"))
        .bind((CHAT_ID, RecordId::from_table_key(CHAT_TB, chat_id.0)));

    simplify_balances(&mut debts);

    for (
        i,
        Debt {
            debtor,
            creditor,
            debt,
        },
    ) in debts.into_iter().enumerate()
    {
        query = query
            .query(format!(
                "RELATE ${DEBTOR}_{i}->{OWES}->${CREDITOR}_{i} 
                SET {AMOUNT} = <decimal> ${AMOUNT}_{i}"
            ))
            .bind((format!("{DEBTOR}_{i}"), debtor))
            .bind((format!("{CREDITOR}_{i}"), creditor))
            .bind((format!("{AMOUNT}_{i}"), debt));
    }

    query = query.query(CommitStatement::default());
    query.await.map(|_| {})
}
