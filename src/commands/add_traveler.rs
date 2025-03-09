use crate::{
    Context,
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::{self, translate_with_args, translate_with_args_default},
    trace_command,
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn add_traveler(
    msg: &Message,
    name: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    if name.is_empty() {
        return Err(CommandError::EmptyInput);
    }

    // Check if traveler exists on db
    let count_res = Traveler::db_count(msg.chat.id, &name).await;
    match count_res {
        Ok(Some(count)) if *count > 0 => {
            tracing::warn!(
                "{}",
                translate_with_args_default(
                    i18n::commands::ADD_TRAVELER_ALREADY_ADDED,
                    &hashmap! {i18n::args::NAME.into() => name.clone().into()},
                )
            );
            Ok(translate_with_args(
                ctx,
                i18n::commands::ADD_TRAVELER_ALREADY_ADDED,
                &hashmap! {i18n::args::NAME.into() => name.into()},
            ))
        }
        Ok(_) => {
            // Create traveler on db
            let create_res = Traveler::db_create(msg.chat.id, &name).await;
            match create_res {
                Ok(_) => {
                    tracing::debug!(DEBUG_SUCCESS);
                    Ok(translate_with_args(
                        ctx,
                        i18n::commands::ADD_TRAVELER_OK,
                        &hashmap! {i18n::args::NAME.into() => name.into()},
                    ))
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
