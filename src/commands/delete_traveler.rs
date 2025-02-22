use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    trace_command,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn delete_traveler(msg: &Message, name: Name) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            // Delete traveler from db
            let delete_res = Traveler::db_delete(msg.chat.id, &name).await;
            match delete_res {
                Ok(_) => {
                    tracing::debug!(DEBUG_SUCCESS);
                    Ok(format!("Traveler {name} deleted successfully."))
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::DeleteTraveler { name })
                }
            }
        }
        Ok(_) => {
            tracing::warn!("Couldn't find traveler {name} to delete.");
            Ok(format!("Couldn't find traveler {name} to delete."))
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::DeleteTraveler {
                name: name.to_owned(),
            })
        }
    }
}
