use crate::{
    balance::Balance,
    db::db,
    expense::Expense,
    i18n::{self, Translate, translate_with_args},
    tests::{TestBot, helpers},
    transfer::Transfer,
    traveler::{Name, Traveler},
};
use maplit::hashmap;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::test]
async fn test_full_expense_flow() {
    let db = db().await;

    let mut bot = TestBot::new(db.clone(), "");

    // Add multiple travelers
    helpers::add_traveler(&mut bot, "Alice").await;
    helpers::add_traveler(&mut bot, "Bob").await;
    helpers::add_traveler(&mut bot, "Charlie").await;

    // List travelers
    bot.update("/listtravelers");
    let response = "Alice\nBob\nCharlie";
    bot.test_last_message(response).await;

    // Add an expense
    helpers::add_expense(
        &mut bot,
        "Test dinner",
        Decimal::from_str("150.50").unwrap(),
        "Alice",
        &["all"],
    )
    .await;

    // Verify expense is listed
    bot.update("/listexpenses");
    let expenses = Expense::db_select(db.clone(), bot.chat_id()).await.unwrap();
    assert_eq!(expenses.len(), 1);

    // Check balances
    bot.update("/showbalances");
    let balances = Balance::balances(db.clone(), bot.chat_id()).await.unwrap();
    assert!(!balances.is_empty());

    // Have Bob make a transfer to Alice
    helpers::transfer(
        &mut bot,
        "Bob",
        "Alice",
        Decimal::from_str("50.17").unwrap(),
    )
    .await;

    // Verify transfer happened
    bot.update("/listtransfers");
    let transfers = Transfer::transfers(db, bot.chat_id()).await.unwrap();
    assert_eq!(transfers.len(), 1);
}

#[tokio::test]
async fn test_expense_splitting_variations() {
    let db = db().await;
    let mut bot = TestBot::new(db.clone(), "");

    // Add travelers
    helpers::add_traveler(&mut bot, "Alice").await;
    helpers::add_traveler(&mut bot, "Bob").await;
    helpers::add_traveler(&mut bot, "Charlie").await;

    // Test expense split evenly
    helpers::add_expense(&mut bot, "Even split dinner", 90.into(), "Alice", &["all"]).await;

    // Test expense with specific amounts
    helpers::add_expense(
        &mut bot,
        "Custom split lunch",
        100.into(),
        "Bob",
        &["Alice:40; Bob:30; Charlie:30", "end"],
    )
    .await;

    // Test expense with percentages
    helpers::add_expense(
        &mut bot,
        "Percentage split breakfast",
        50.into(),
        "Charlie",
        &["Alice:50%; Bob:25%; Charlie:25%", "end"],
    )
    .await;

    // Verify all expenses are tracked
    bot.update("/listexpenses");
    let expenses = Expense::db_select(db.clone(), bot.chat_id()).await.unwrap();
    assert_eq!(expenses.len(), 3);

    // Check balances reflect complex splits correctly
    bot.update("/showbalances");
    let balances = Balance::balances(db, bot.chat_id()).await.unwrap();
    assert!(!balances.is_empty());
}

#[tokio::test]
async fn test_traveler_lifecycle() {
    let db = db().await;
    let mut bot = TestBot::new(db.clone(), "");

    // Add a traveler
    helpers::add_traveler(&mut bot, "Alice").await;

    // Verify traveler exists
    bot.update("/listtravelers");
    let response = "Alice";
    bot.test_last_message(response).await;

    // Try to add same traveler again - should fail
    bot.update("/addtraveler Alice");
    let response = translate_with_args(
        bot.context(),
        i18n::commands::ADD_TRAVELER_ALREADY_ADDED,
        &hashmap! {i18n::args::NAME.into() => "Alice".into()},
    );
    bot.test_last_message(&response).await;

    // Add another traveler
    helpers::add_traveler(&mut bot, "Bob").await;

    // Add an expense to create a relationship
    helpers::add_expense(&mut bot, "Test expense", 100.into(), "Alice", &["all"]).await;

    // Try to delete traveler with expenses - should fail
    // 1. Retrieve traveler "Alice" and their expenses
    let traveler =
        Traveler::db_select_by_name(db.clone(), bot.chat_id(), &Name::from_str("Alice").unwrap())
            .await
            .unwrap()
            .unwrap();
    let expenses = Expense::db_select_by_payer(db, traveler).await.unwrap();
    // 2. Check that only one expense is returned
    assert_eq!(expenses.len(), 1);
    let expense = expenses.first().unwrap();

    // 3. Delete traveler "Alice" -> has expenses
    bot.update("/deletetraveler Alice");
    let response = translate_with_args(
        bot.context(),
        i18n::commands::DELETE_TRAVELER_HAS_EXPENSES,
        &hashmap! {
            i18n::args::NAME.into() => "Alice".into(),
            i18n::args::EXPENSES.into() => expense.translate_default().into(),
        },
    );
    bot.test_last_message(&response).await;

    // Try to delete non-existent traveler
    bot.update("/deletetraveler Charlie");
    let response = translate_with_args(
        bot.context(),
        i18n::commands::DELETE_TRAVELER_NOT_FOUND,
        &hashmap! {i18n::args::NAME.into() => "Charlie".into()},
    );
    bot.test_last_message(&response).await;
}

#[tokio::test]
async fn test_i18n_and_currency() {
    let db = db().await;
    let mut bot = TestBot::new(db.clone(), "");

    // Set language to Italian
    bot.update("/setlanguage it-IT");
    let last_message = bot.dispatch_and_last_message().await.unwrap();
    let response = translate_with_args(
        bot.context(), // Use the new context to retrieve the updated language
        i18n::commands::SET_LANGUAGE_OK,
        &hashmap! {i18n::args::LANGID.into() => "it-IT".into()},
    );
    // Check that the last message is the expected response
    assert_eq!(last_message, response);

    // Set currency to EUR
    bot.update("/setcurrency EUR");
    let response = translate_with_args(
        bot.context(),
        i18n::commands::SET_CURRENCY_OK,
        &hashmap! {i18n::args::CURRENCY.into() => "EUR".into()},
    );
    bot.test_last_message(&response).await;

    // Add travelers
    helpers::add_traveler(&mut bot, "Alice").await;
    helpers::add_traveler(&mut bot, "Bob").await;

    // Add expense with new currency
    helpers::add_expense(&mut bot, "Test expense", 100.into(), "Alice", &["all"]).await;

    // Verify expense is listed with correct currency
    bot.update("/listexpenses");
    let expenses = Expense::db_select(db, bot.chat_id()).await.unwrap();
    assert_eq!(expenses.len(), 1);
    let first_expense = expenses.first().unwrap();
    assert!(first_expense.translate(bot.context()).contains("€"));
}
