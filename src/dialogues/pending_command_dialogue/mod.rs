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
pub mod delete_expense;
pub mod delete_transfer;
pub mod delete_traveler;
pub mod set_currency;
pub mod set_language;
pub mod show_expense;

use add_traveler::AddTravelerState;
use delete_expense::DeleteExpenseState;
use delete_transfer::DeleteTransferState;
use delete_traveler::DeleteTravelerState;
use set_currency::SetCurrencyState;
use set_language::SetLanguageState;
use show_expense::ShowExpenseState;
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
    DeleteTraveler(DeleteTravelerState),
    DeleteExpense(DeleteExpenseState),
    ShowExpense(ShowExpenseState),
    DeleteTransfer(DeleteTransferState),
    SetLanguage(SetLanguageState),
    SetCurrency(SetCurrencyState),
}

pub type PendingCommandStorage = InMemStorage<PendingCommandState>;
pub type PendingCommandDialogue = Dialogue<PendingCommandState, PendingCommandStorage>;

/// Maps a [`PendingCommandState`] to the Fluent message id describing the
/// underlying command for user-facing messages (e.g. the "another process
/// is already running" notice). `Start` reports a generic placeholder
/// because no specific command has been picked yet.
impl crate::dialogues::storage::DialogueState for PendingCommandState {
    fn running_label(&self) -> &'static str {
        use crate::i18n::commands::*;
        match self {
            PendingCommandState::Start => RUNNING_PROCESS_UNKNOWN,
            PendingCommandState::AddTraveler(_) => RUNNING_PROCESS_ADD_TRAVELER,
            PendingCommandState::DeleteTraveler(_) => RUNNING_PROCESS_DELETE_TRAVELER,
            PendingCommandState::DeleteExpense(_) => RUNNING_PROCESS_DELETE_EXPENSE,
            PendingCommandState::ShowExpense(_) => RUNNING_PROCESS_SHOW_EXPENSE,
            PendingCommandState::DeleteTransfer(_) => RUNNING_PROCESS_DELETE_TRANSFER,
            PendingCommandState::SetLanguage(_) => RUNNING_PROCESS_SET_LANGUAGE,
            PendingCommandState::SetCurrency(_) => RUNNING_PROCESS_SET_CURRENCY,
        }
    }
}

/// Returns the dispatcher subtree that drives every pending-command dialogue.
/// Composed into [`crate::handler_tree`] alongside other dialogues' branches.
pub fn handler_branch() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use PendingCommandState::*;
    use teloxide::dptree::{self, case};

    dptree::entry()
        // Only enter this subtree if a pending-command dialogue is active.
        .filter_async(crate::dialogues::storage::is_running::<PendingCommandState>)
        .enter_dialogue::<Message, PendingCommandStorage, PendingCommandState>()
        .branch(
            case![AddTraveler(state)]
                .branch(case![AddTravelerState::AskName].endpoint(add_traveler::receive_name)),
        )
        .branch(
            case![DeleteTraveler(state)].branch(
                case![DeleteTravelerState::AskName].endpoint(delete_traveler::receive_name),
            ),
        )
        .branch(
            case![DeleteExpense(state)].branch(
                case![DeleteExpenseState::AskNumber].endpoint(delete_expense::receive_number),
            ),
        )
        .branch(
            case![ShowExpense(state)]
                .branch(case![ShowExpenseState::AskNumber].endpoint(show_expense::receive_number)),
        )
        .branch(case![DeleteTransfer(state)].branch(
            case![DeleteTransferState::AskNumber].endpoint(delete_transfer::receive_number),
        ))
        .branch(
            case![SetLanguage(state)]
                .branch(case![SetLanguageState::AskLangid].endpoint(set_language::receive_langid)),
        )
        .branch(
            case![SetCurrency(state)].branch(
                case![SetCurrencyState::AskCurrency].endpoint(set_currency::receive_currency),
            ),
        )
}
