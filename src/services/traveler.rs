use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};
use teloxide::types::ChatId;

use crate::tables::expense::Expense;
use crate::tables::traveler::{Name, Traveler};
use crate::utils::update_debts;

use super::ServiceError;

/// Add a traveler to a chat. Fails if the name already exists.
pub async fn add_traveler(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    name: &Name,
) -> Result<(), ServiceError> {
    let count = Traveler::db_count(db.clone(), chat_id, name).await?;
    if count.map(|c| *c > 0).unwrap_or(false) {
        return Err(ServiceError::AlreadyExists("Traveler"));
    }

    Traveler::db_create(db, chat_id, name).await?;
    Ok(())
}

/// Delete a traveler from a chat.
///
/// Fails if the traveler does not exist or has associated expenses.
pub async fn delete_traveler(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    name: &Name,
) -> Result<(), ServiceError> {
    let traveler = Traveler::db_select_by_name(db.clone(), chat_id, name)
        .await?
        .ok_or(ServiceError::NotFound("Traveler"))?;

    let expenses = Expense::db_select_by_payer(db.clone(), traveler).await?;
    if !expenses.is_empty() {
        return Err(ServiceError::HasAssociatedExpenses(expenses));
    }

    Traveler::db_delete(db.clone(), chat_id, name).await?;

    if let Err(e) = update_debts(db, chat_id).await {
        tracing::warn!("update_debts after delete traveler failed: {e}");
    }

    Ok(())
}
