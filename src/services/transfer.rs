use rust_decimal::Decimal;
use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};
use teloxide::types::ChatId;

use crate::relationships::transferred_to::TransferredTo;
use crate::tables::traveler::{Name, Traveler};
use crate::utils::update_debts;

use super::ServiceError;

/// Create a transfer between two travelers and update debts.
///
/// Returns the created `TransferredTo` relation.
pub async fn create_transfer(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    from: &Name,
    to: &Name,
    amount: Decimal,
) -> Result<TransferredTo, ServiceError> {
    let sender = Traveler::db_select_by_name(db.clone(), chat_id, from)
        .await?
        .ok_or(ServiceError::NotFound("Sender"))?;

    let receiver = Traveler::db_select_by_name(db.clone(), chat_id, to)
        .await?
        .ok_or(ServiceError::NotFound("Receiver"))?;

    let transfer = TransferredTo::db_relate(db.clone(), amount, sender.id, receiver.id)
        .await?
        .ok_or(ServiceError::NotFound("Transfer"))?;

    if let Err(e) = update_debts(db, chat_id).await {
        tracing::warn!("update_debts after add transfer failed: {e}");
    }

    Ok(transfer)
}

/// Delete a transfer by number and update debts.
pub async fn delete_transfer(
    db: Arc<Surreal<Any>>,
    chat_id: ChatId,
    number: i64,
) -> Result<(), ServiceError> {
    let count = TransferredTo::db_count(db.clone(), chat_id, number).await?;
    if !count.map(|c| *c > 0).unwrap_or(false) {
        return Err(ServiceError::NotFound("Transfer"));
    }

    TransferredTo::db_delete(db.clone(), chat_id, number).await?;

    if let Err(e) = update_debts(db, chat_id).await {
        tracing::warn!("update_debts after delete transfer failed: {e}");
    }

    Ok(())
}
