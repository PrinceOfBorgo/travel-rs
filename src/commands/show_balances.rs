use crate::{
    Context,
    balance::Balance,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{
        self, Translate, TranslateWithArgs,
        args::{TRAVELER_IS_CASE_CREDITOR, TRAVELER_IS_CASE_DEBTOR},
    },
    money_wrapper::MoneyWrapper,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command_db)]
pub async fn show_balances(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Option<Name>,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!("{LOG_DEBUG_START}");
    let res = match &name {
        None => show_balances_no_name(db, msg, ctx.clone()).await,
        Some(name) => {
            // Look up the traveler so we can use the canonical name (as
            // stored in the database) in the response and in any further
            // comparisons.
            let select_res = Traveler::db_select_by_name(db.clone(), msg.chat.id, name).await;
            match select_res {
                Ok(Some(traveler)) => {
                    show_balances_with_name(db, msg, traveler.name, ctx.clone()).await
                }
                Ok(_) => {
                    tracing::warn!(
                        "{}",
                        i18n::commands::SHOW_BALANCES_TRAVELER_NOT_FOUND
                            .translate_with_args_default(
                                &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                            )
                    );
                    return Ok(i18n::commands::SHOW_BALANCES_TRAVELER_NOT_FOUND
                        .translate_with_args(
                            ctx,
                            &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                        ));
                }
                Err(err) => {
                    tracing::error!("{err}");
                    return Err(CommandError::ShowBalances { name: name.clone() });
                }
            }
        }
    };

    match res {
        Ok(reply) => {
            tracing::debug!("{LOG_DEBUG_SUCCESS}");
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ShowBalances {
                name: name.unwrap_or_default(),
            })
        }
    }
}

async fn show_balances_no_name(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, surrealdb::Error> {
    let list_res = Balance::balances(db, msg.chat.id).await;
    match list_res {
        Ok(balances) => {
            let currency = ctx.lock().expect("Failed to lock context").currency.clone();
            let mut any_nonzero = false;
            let formatted_balances = balances
                .into_iter()
                .filter_map(
                    |Balance {
                         debtor_name,
                         creditor_name,
                         debt,
                         ..
                     }| {
                        let debt = MoneyWrapper::new(debt, &currency);
                        if debt.round_value().is_zero() {
                            None
                        } else {
                            any_nonzero = true;
                            Some(i18n::commands::SHOW_BALANCES_OK.translate_with_args(
                                ctx.clone(),
                                &hashmap! {
                                    i18n::args::DEBTOR.into() => debtor_name.into(),
                                    i18n::args::DEBT.into() => debt.to_string().into(),
                                    i18n::args::CREDITOR.into() => creditor_name.into(),
                                },
                            ))
                        }
                    },
                )
                .collect::<Vec<_>>();

            let reply = if any_nonzero && !formatted_balances.is_empty() {
                formatted_balances.join("\n")
            } else {
                // If there are no balances or all are zero after rounding, show a special message
                i18n::commands::SHOW_BALANCES_SETTLED_UP.translate(ctx)
            };
            Ok(reply)
        }
        Err(err) => Err(err),
    }
}

pub async fn show_balances_with_name(
    db: Arc<Surreal<Any>>,
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, surrealdb::Error> {
    // Retrieve balances from db
    let list_res = Balance::balances_by_name(db, msg.chat.id, name.to_owned()).await;
    match list_res {
        Ok(balances) => {
            let currency = ctx.lock().expect("Failed to lock context").currency.clone();
            let mut any_nonzero = false;
            let formatted_balances = balances
                .into_iter()
                .filter_map(
                    |Balance {
                        debtor_name,
                        creditor_name,
                        debt,
                        ..
                    }| {
                        let debt = MoneyWrapper::new(debt, &currency);
                        if debt.round_value().is_zero() {
                            None
                        } else {
                            any_nonzero = true;
                            Some(i18n::commands::SHOW_BALANCES_TRAVELER_OK.translate_with_args(
                                ctx.clone(),
                                &hashmap! {
                                    i18n::args::TRAVELER_NAME.into() => name.clone().into(),
                                    i18n::args::TRAVELER_IS.into() => if debtor_name == name { TRAVELER_IS_CASE_DEBTOR } else { TRAVELER_IS_CASE_CREDITOR }.into(),
                                    i18n::args::DEBT.into() => debt.to_string().into(),
                                    i18n::args::OTHER_TRAVELER_NAME.into() => if debtor_name == name { creditor_name } else { debtor_name }.into(),
                                }
                            ))
                        }
                    },
                )
                .collect::<Vec<_>>();

            let reply = if any_nonzero && !formatted_balances.is_empty() {
                formatted_balances.join("\n")
            } else {
                // If there are no balances or all are zero after rounding, show a special message
                i18n::commands::SHOW_BALANCES_TRAVELER_SETTLED_UP
                    .translate_with_args(ctx, &hashmap! {i18n::args::NAME.into() => name.into()})
            };
            Ok(reply)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        balance::Balance,
        db::db,
        i18n::{
            self, Translate, TranslateWithArgs,
            args::{TRAVELER_IS_CASE_CREDITOR, TRAVELER_IS_CASE_DEBTOR},
        },
        money_wrapper::MoneyWrapper,
        tests::{TestBot, helpers},
        traveler::Name,
    };
    use maplit::hashmap;
    use rust_decimal::Decimal;

    test! { show_balances_no_name,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob", "Charlie" and "David"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;
        helpers::add_traveler(&mut bot, "David").await;

        // Add expense #1
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            Decimal::from_str("100.8").unwrap(),
            "Alice",
            &["all"]
        ).await;

        // Add expense #2
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Bob",
            &["Alice:100", "end"]
        ).await;

        // Add expense #3
        helpers::add_expense(
            &mut bot,
            "Test expense 3",
            100.into(),
            "Charlie",
            &["Alice: 40; Bob: 40%; Charlie; David", "end"]
        ).await;

        // Add expense #4
        helpers::add_expense(
            &mut bot,
            "Test expense 4",
            100.into(),
            "David",
            &["Alice: 50; Bob: 50", "end"]
        ).await;

        // Add expense #5
        helpers::add_expense(
            &mut bot,
            "Test expense 5",
            100.into(),
            "Alice",
            &["Alice: 50", "all"]
        ).await;

        // Add expense #6
        helpers::add_expense(
            &mut bot,
            "Test expense 6",
            100.into(),
            "Bob",
            &["Alice: 67%; Bob: 34%", "end"]
        ).await;

        // Add expense #7
        helpers::add_expense(
            &mut bot,
            "Test expense 7",
            100.into(),
            "Charlie",
            &["Alice: 110", "end", "Alice:100", "end"]
        ).await;

        // Transfer money
        helpers::transfer(&mut bot, "Alice", "Bob", Decimal::from_str("24.4").unwrap()).await;
        helpers::transfer(&mut bot, "Charlie", "Alice", Decimal::from_str("25.2").unwrap()).await;
        helpers::transfer(&mut bot, "David", "Bob", Decimal::from_str("25.2").unwrap()).await;
        helpers::transfer(&mut bot, "Bob", "Charlie", Decimal::from_str("-25.2").unwrap()).await;

        // Show balances
        bot.update("/showbalances");

        let balances = Balance::balances(db, bot.chat_id()).await.unwrap();
        let ctx = bot.context().clone();
        let currency = ctx.lock().expect("Failed to lock context").currency.clone();
        let response = balances
            .into_iter()
            .filter_map(
                |Balance {
                    debtor_name,
                    creditor_name,
                    debt,
                    ..
                }| {
                    let debt = MoneyWrapper::new(debt, &currency);
                    if debt.round_value().is_zero() {
                        None
                    } else {
                        Some(i18n::commands::SHOW_BALANCES_OK.translate_with_args(
                            ctx.clone(),
                            &hashmap! {
                                i18n::args::DEBTOR.into() => debtor_name.into(),
                                i18n::args::DEBT.into() => debt.to_string().into(),
                                i18n::args::CREDITOR.into() => creditor_name.into(),
                            },
                        ))
                    }
                },
            )
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;
    }

    test! { show_balances_settled_up,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob", "Charlie" and "David"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;
        helpers::add_traveler(&mut bot, "David").await;

        // Add expense #1
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            Decimal::from_str("100.8").unwrap(),
            "Alice",
            &["all"]
        ).await;

        // Add expense #2
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Bob",
            &["Alice:100", "end"]
        ).await;

        // Add expense #3
        helpers::add_expense(
            &mut bot,
            "Test expense 3",
            100.into(),
            "Charlie",
            &["Alice: 40; Bob: 40%; Charlie; David", "end"]
        ).await;

        // Add expense #4
        helpers::add_expense(
            &mut bot,
            "Test expense 4",
            100.into(),
            "David",
            &["Alice: 50; Bob: 50", "end"]
        ).await;

        // Add expense #5
        helpers::add_expense(
            &mut bot,
            "Test expense 5",
            100.into(),
            "Alice",
            &["Alice: 50", "all"]
        ).await;

        // Add expense #6
        helpers::add_expense(
            &mut bot,
            "Test expense 6",
            100.into(),
            "Bob",
            &["Alice: 67%; Bob: 34%", "end"]
        ).await;

        // Add expense #7
        helpers::add_expense(
            &mut bot,
            "Test expense 7",
            100.into(),
            "Charlie",
            &["Alice: 110", "end", "Alice:100", "end"]
        ).await;

        // Transfer money
        helpers::transfer(&mut bot, "Alice", "Charlie", Decimal::from_str("140.13").unwrap()).await;
        helpers::transfer(&mut bot, "Alice", "Bob", Decimal::from_str("51.13").unwrap()).await;
        helpers::transfer(&mut bot, "Alice", "David", Decimal::from_str("40.13").unwrap()).await;

        // Show balances
        bot.update("/showbalances");
        let response = i18n::commands::SHOW_BALANCES_SETTLED_UP.translate_default();
        bot.test_last_message(&response).await;
    }

    test! { show_balances_with_name,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob", "Charlie" and "David"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;
        helpers::add_traveler(&mut bot, "David").await;

        // Add expense #1
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            Decimal::from_str("100").unwrap(),
            "Alice",
            &["Bob", "end"]
        ).await;

        // Add expense #2
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Bob",
            &["Charlie:100", "end"]
        ).await;

        // Add expense #3
        helpers::add_expense(
            &mut bot,
            "Test expense 3",
            100.into(),
            "Charlie",
            &["Alice: 40; Bob: 40%; Charlie", "end"]
        ).await;

        // Transfer money
        helpers::transfer(&mut bot, "David", "Alice", Decimal::from_str("100").unwrap()).await;
        helpers::transfer(&mut bot, "Charlie", "David", Decimal::from_str("50").unwrap()).await;
        helpers::transfer(&mut bot, "David", "Bob", Decimal::from_str("75").unwrap()).await;
        helpers::transfer(&mut bot, "Alice", "Charlie", Decimal::from_str("50").unwrap()).await;
        helpers::transfer(&mut bot, "Alice", "Bob", Decimal::from_str("10").unwrap()).await;
        helpers::transfer(&mut bot, "Bob", "Charlie", Decimal::from_str("50").unwrap()).await;

        // Show Alice balances
        let name = "Alice";
        bot.update(&format!("/showbalances {name}"));

        let balances = Balance::balances_by_name(db.clone(), bot.chat_id(), Name::from_str(name).unwrap()).await.unwrap();
        let ctx = bot.context().clone();
        let currency = ctx.lock().expect("Failed to lock context").currency.clone();
        let response = balances
            .into_iter()
            .filter_map(
                |Balance {
                    debtor_name,
                    creditor_name,
                    debt,
                    ..
                }| {
                    let debt = MoneyWrapper::new(debt, &currency);
                    if debt.round_value().is_zero() {
                        None
                    } else {
                        Some(i18n::commands::SHOW_BALANCES_TRAVELER_OK.translate_with_args(
                            ctx.clone(),
                            &hashmap! {
                                i18n::args::TRAVELER_NAME.into() => name.into(),
                                i18n::args::TRAVELER_IS.into() => if &*debtor_name == name { TRAVELER_IS_CASE_DEBTOR } else { TRAVELER_IS_CASE_CREDITOR }.into(),
                                i18n::args::DEBT.into() => debt.to_string().into(),
                                i18n::args::OTHER_TRAVELER_NAME.into() => if &*debtor_name == name { creditor_name } else { debtor_name }.into(),
                            },
                        ))
                    }
                },
            )
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;

        // Show Bob balances
        let name = "Bob";
        bot.update(&format!("/showbalances {name}"));

        let balances = Balance::balances_by_name(db.clone(), bot.chat_id(), Name::from_str(name).unwrap()).await.unwrap();
        let ctx = bot.context().clone();
        let currency = ctx.lock().expect("Failed to lock context").currency.clone();
        let response = balances
            .into_iter()
            .filter_map(
                |Balance {
                    debtor_name,
                    creditor_name,
                    debt,
                    ..
                }| {
                    let debt = MoneyWrapper::new(debt, &currency);
                    if debt.round_value().is_zero() {
                        None
                    } else {
                        Some(i18n::commands::SHOW_BALANCES_TRAVELER_OK.translate_with_args(
                            ctx.clone(),
                            &hashmap! {
                                i18n::args::TRAVELER_NAME.into() => name.into(),
                                i18n::args::TRAVELER_IS.into() => if &*debtor_name == name { TRAVELER_IS_CASE_DEBTOR } else { TRAVELER_IS_CASE_CREDITOR }.into(),
                                i18n::args::DEBT.into() => debt.to_string().into(),
                                i18n::args::OTHER_TRAVELER_NAME.into() => if &*debtor_name == name { creditor_name } else { debtor_name }.into(),
                            },
                        ))
                    }
                },
            )
            .collect::<Vec<_>>()
            .join("\n");
        bot.test_last_message(&response).await;
    }

    test! { show_balances_traveler_settled_up,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice", "Bob", "Charlie" and "David"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;
        helpers::add_traveler(&mut bot, "Charlie").await;
        helpers::add_traveler(&mut bot, "David").await;

        // Add expense #1
        helpers::add_expense(
            &mut bot,
            "Test expense 1",
            Decimal::from_str("100").unwrap(),
            "Alice",
            &["Bob", "end"]
        ).await;

        // Add expense #2
        helpers::add_expense(
            &mut bot,
            "Test expense 2",
            100.into(),
            "Bob",
            &["Charlie:100", "end"]
        ).await;

        // Add expense #3
        helpers::add_expense(
            &mut bot,
            "Test expense 3",
            100.into(),
            "Charlie",
            &["Alice: 40; Bob: 40%; Charlie", "end"]
        ).await;

        // Transfer money
        helpers::transfer(&mut bot, "David", "Alice", Decimal::from_str("100").unwrap()).await;
        helpers::transfer(&mut bot, "Charlie", "David", Decimal::from_str("50").unwrap()).await;
        helpers::transfer(&mut bot, "David", "Bob", Decimal::from_str("75").unwrap()).await;
        helpers::transfer(&mut bot, "Alice", "Charlie", Decimal::from_str("50").unwrap()).await;
        helpers::transfer(&mut bot, "Bob", "Alice", Decimal::from_str("10").unwrap()).await;
        helpers::transfer(&mut bot, "Bob", "Charlie", Decimal::from_str("50").unwrap()).await;

        // Show balances
        bot.update("/showbalances Alice");
        let response = i18n::commands::SHOW_BALANCES_TRAVELER_SETTLED_UP.translate_with_args_default(&hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { show_balances_traveler_not_found,
        let db = db().await;

        let mut bot = TestBot::new(db.clone(), "/showbalances UnknownTraveler");
        let response = i18n::commands::SHOW_BALANCES_TRAVELER_NOT_FOUND.translate_with_args_default(&hashmap! {i18n::args::NAME.into() => "UnknownTraveler".into()},
        );
        bot.test_last_message(&response).await;
    }

    test! { show_balances_uses_canonical_name_case_insensitive,
        let db = db().await;
        let mut bot = TestBot::new(db.clone(), "");

        // Add travelers "Alice" and "Bob"
        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Bob owes Alice 50
        helpers::add_expense(
            &mut bot,
            "Test expense",
            100.into(),
            "Alice",
            &["Alice; Bob", "end"],
        ).await;

        // Query with mixed casing -> should display canonical "Alice"
        // (not "ALICE") and find the balance with Bob.
        bot.update("/showbalances ALICE");
        let last = bot.dispatch_and_last_message().await.unwrap_or_default();
        assert!(
            last.contains("Alice") && !last.contains("ALICE"),
            "Expected canonical name 'Alice' in response, got: {last}"
        );
        // Sanity-check: not the "settled up" message.
        let settled = i18n::commands::SHOW_BALANCES_TRAVELER_SETTLED_UP
            .translate_with_args_default(&hashmap! {i18n::args::NAME.into() => "Alice".into()});
        assert_ne!(last, settled);
    }
}
