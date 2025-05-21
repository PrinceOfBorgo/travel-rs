use crate::debt::Debt;
use macro_rules_attribute::attribute_alias;
use rust_decimal::Decimal;
use std::{collections::HashMap, sync::Arc};
use surrealdb::{
    RecordId, Surreal,
    engine::any::Any,
    sql::statements::{BeginStatement, CommitStatement},
};
use teloxide::types::ChatId;

attribute_alias! {
    #[apply(trace_skip_all)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip_all,
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_command)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(msg, ctx),
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_state)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(bot, dialogue, msg),
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id
        )
    )
    ];
}

const FN_CALC_DEBTS: &str = "fn::calc_debts";
const FN_CLEAR_DEBTS: &str = "fn::clear_debts";

fn simplify_balances(debts: &mut Vec<Debt>) {
    // Create a HashMap to store the original RecordId for each participant
    // This avoids interior mutability issues by using String as the key
    let keys: HashMap<String, RecordId> = debts
        .iter()
        .flat_map(|d| {
            vec![
                (d.debtor.to_string(), d.debtor.clone()),
                (d.creditor.to_string(), d.creditor.clone()),
            ]
        })
        .collect();

    // Create a HashMap to store the net balance of each participant
    let mut balance_map: HashMap<String, Decimal> = HashMap::new();

    // Calculate the net balance for each participant
    for debt in debts.iter() {
        // Subtract the debt amount from the debtor's balance
        *balance_map
            .entry(debt.debtor.to_string())
            .or_insert(Decimal::ZERO) -= debt.debt;
        // Add the debt amount to the creditor's balance
        *balance_map
            .entry(debt.creditor.to_string())
            .or_insert(Decimal::ZERO) += debt.debt;
    }

    // Separate participants into creditors and debtors
    let mut creditors: Vec<_> = balance_map
        .clone()
        .into_iter()
        .filter(|&(_, v)| v > Decimal::ZERO)
        .collect();
    let mut debtors: Vec<_> = balance_map
        .into_iter()
        .filter(|&(_, v)| v < Decimal::ZERO)
        .collect();

    // Sort creditors in descending order of their balances
    creditors.sort_by(|a, b| b.1.cmp(&a.1));
    // Sort debtors in ascending order of their balances
    debtors.sort_by(|a, b| a.1.cmp(&b.1));

    // Clear the original list of debts
    debts.clear();

    // Simplify the balances by creating new debt transactions
    while let (Some((debtor, debtor_amount)), Some((creditor, creditor_amount))) =
        (debtors.last(), creditors.last())
    {
        // Determine the amount to be transferred
        let amount = debtor_amount.abs().min(*creditor_amount);
        // Retrieve the original RecordId for the debtor and creditor
        let debtor = keys
            .get(debtor)
            .expect("Debtor with id {debtor} should exist")
            .clone();
        let creditor = keys
            .get(creditor)
            .expect("Creditor with id {creditor} should exist")
            .clone();

        // Create a new debt transaction
        debts.push(Debt {
            debtor,
            creditor,
            debt: amount,
        });

        // Update the debtor's balance
        if debtor_amount + amount == Decimal::ZERO {
            debtors.pop();
        } else if let Some((_, amt)) = debtors.last_mut() {
            *amt += amount;
        }

        // Update the creditor's balance
        if creditor_amount - amount == Decimal::ZERO {
            creditors.pop();
        } else if let Some((_, amt)) = creditors.last_mut() {
            *amt -= amount;
        }
    }
}

pub async fn update_debts(db: Arc<Surreal<Any>>, chat_id: ChatId) -> Result<(), surrealdb::Error> {
    use crate::{
        chat::{ID as CHAT_ID, TABLE as CHAT_TB},
        debt::{CREDITOR, DEBT, DEBTOR},
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
            )) // Define the new relationship
            .bind((format!("{DEBTOR}_{i}"), debtor))
            .bind((format!("{CREDITOR}_{i}"), creditor))
            .bind((format!("{AMOUNT}_{i}"), debt));
    }

    query = query.query(CommitStatement::default());
    query.await.map(|_| {})
}
