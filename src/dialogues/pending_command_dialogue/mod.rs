//! Dialogues used to collect missing arguments for parameterized commands.
//!
//! When the user invokes a command that requires parameters but some are
//! missing, we start a dialogue that prompts the user for each missing
//! argument and then runs the command with the collected values.
//!
//! Each command that opts in defines its own state enum and handlers in a
//! dedicated submodule. The outer [`PendingCommandState`] wraps those
//! per-command states so that a single shared storage can drive any of them.

pub mod add_traveler;

use add_traveler::AddTravelerState;
use teloxide::{
    dispatching::{
        HandlerExt, UpdateHandler,
        dialogue::{Dialogue, InMemStorage},
    },
    types::Message,
};

#[derive(Debug, Clone, Default)]
pub enum PendingCommandState {
    #[default]
    Start,
    AddTraveler(AddTravelerState),
}

pub type PendingCommandStorage = InMemStorage<PendingCommandState>;
pub type PendingCommandDialogue = Dialogue<PendingCommandState, PendingCommandStorage>;

/// Returns the dispatcher subtree that drives every pending-command dialogue.
/// Composed into [`crate::handler_tree`] alongside other dialogues' branches.
pub fn handler_branch() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use AddTravelerState::*;
    use PendingCommandState::*;
    use teloxide::dptree::{self, case};

    dptree::entry()
        // Only enter this subtree if a pending-command dialogue is active.
        .filter_async(crate::dialogues::storage::is_running::<PendingCommandState>)
        .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
        .branch(
            case![AddTraveler(state)].branch(case![AskName].endpoint(add_traveler::receive_name)),
        )
}
