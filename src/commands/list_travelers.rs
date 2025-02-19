use crate::{errors::CommandError, trace_command, traveler::Traveler};
use macro_rules_attribute::apply;
use teloxide::prelude::*;
use tracing::Level;

#[apply(trace_command)]
pub async fn list_travelers(msg: &Message) -> Result<String, CommandError> {
    tracing::debug!("START");
    let list_res = Traveler::db_select(msg.chat.id).await;
    match list_res {
        Ok(travelers) => {
            let reply = if travelers.is_empty() {
                format!(
                    "No travelers found. Use `/{add_traveler} <name>` to add one.",
                    add_traveler = variant_to_string!(Command::AddTraveler)
                )
            } else {
                travelers
                    .into_iter()
                    .map(|traveler| (*traveler.name).to_owned())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            tracing::debug!("SUCCESS");
            Ok(reply)
        }
        Err(err) => {
            tracing::error!("{err}");
            Err(CommandError::ListTravelers)
        }
    }
}
