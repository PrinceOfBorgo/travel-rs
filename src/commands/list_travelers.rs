use crate::{
    consts::{DEBUG_START, DEBUG_SUCCESS},
    errors::CommandError,
    i18n::translate,
    trace_command,
    traveler::Traveler,
};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_travelers(msg: &Message) -> Result<String, CommandError> {
    tracing::debug!(DEBUG_START);
    let list_res = Traveler::db_select(msg.chat.id).await;
    match list_res {
        Ok(travelers) => {
            let reply = if travelers.is_empty() {
                translate(msg.chat.id, "i18n-list-travelers-not-found").await
            } else {
                travelers
                    .into_iter()
                    .map(|traveler| (*traveler.name).to_owned())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!(DEBUG_SUCCESS);
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListTravelers)
        }
    }
}
