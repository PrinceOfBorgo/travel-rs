use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::translate_with_args,
    trace_command,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn add_traveler(msg: &Message, name: Name) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            tracing::warn!("Traveler {name} has already been added to the travel plan.");
            Ok(translate_with_args(
                msg.chat.id,
                "i18n-add-traveler-already-added",
                &hashmap! {"name".into() => name.into()},
            )
            .await)
        }
        Ok(_) => {
            // Create traveler on db
            let create_res = Traveler::db_create(msg.chat.id, &name).await;
            match create_res {
                Ok(_) => {
                    tracing::debug!(DEBUG_SUCCESS);
                    Ok(translate_with_args(
                        msg.chat.id,
                        "i18n-add-traveler-ok",
                        &hashmap! {"name".into() => name.into()},
                    )
                    .await)
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(CommandError::AddTraveler { name })
                }
            }
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::AddTraveler {
                name: name.to_owned(),
            })
        }
    }
}
