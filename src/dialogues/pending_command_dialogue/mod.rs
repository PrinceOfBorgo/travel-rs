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
pub mod clear_all;
pub mod clear_expenses;
pub mod clear_transfers;
pub mod clear_travelers;
pub mod delete_expense;
pub mod delete_transfer;
pub mod delete_traveler;
pub mod list_expenses;
pub mod set_currency;
pub mod set_language;
pub mod show_expense;
pub mod transfer;

use add_traveler::AddTravelerState;
use clear_all::ClearAllState;
use clear_expenses::ClearExpensesState;
use clear_transfers::ClearTransfersState;
use clear_travelers::ClearTravelersState;
use delete_expense::DeleteExpenseState;
use delete_transfer::DeleteTransferState;
use delete_traveler::DeleteTravelerState;
use list_expenses::ListExpensesState;
use set_currency::SetCurrencyState;
use set_language::SetLanguageState;
use show_expense::ShowExpenseState;
use teloxide::{
    dispatching::{
        HandlerExt, UpdateHandler,
        dialogue::{Dialogue, InMemStorage},
    },
    types::{CallbackQuery, Message},
};
use transfer::TransferState;

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
    ListExpenses(ListExpensesState),
    Transfer(TransferState),
    ClearTravelers(ClearTravelersState),
    ClearExpenses(ClearExpensesState),
    ClearTransfers(ClearTransfersState),
    ClearAll(ClearAllState),
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
            PendingCommandState::ListExpenses(_) => RUNNING_PROCESS_LIST_EXPENSES,
            PendingCommandState::Transfer(_) => RUNNING_PROCESS_TRANSFER,
            PendingCommandState::ClearTravelers(_) => RUNNING_PROCESS_CLEAR_TRAVELERS,
            PendingCommandState::ClearExpenses(_) => RUNNING_PROCESS_CLEAR_EXPENSES,
            PendingCommandState::ClearTransfers(_) => RUNNING_PROCESS_CLEAR_TRANSFERS,
            PendingCommandState::ClearAll(_) => RUNNING_PROCESS_CLEAR_ALL,
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
            case![DeleteTraveler(state)]
                .branch(case![DeleteTravelerState::AskName].endpoint(delete_traveler::receive_name))
                .branch(
                    case![DeleteTravelerState::Confirm(name)]
                        .endpoint(delete_traveler::receive_confirm_text),
                ),
        )
        .branch(
            case![DeleteExpense(state)]
                .branch(
                    case![DeleteExpenseState::AskNumber].endpoint(delete_expense::receive_number),
                )
                .branch(
                    case![DeleteExpenseState::Confirm(number)]
                        .endpoint(delete_expense::receive_confirm_text),
                ),
        )
        .branch(
            case![ShowExpense(state)]
                .branch(case![ShowExpenseState::AskNumber].endpoint(show_expense::receive_number)),
        )
        .branch(
            case![DeleteTransfer(state)]
                .branch(
                    case![DeleteTransferState::AskNumber].endpoint(delete_transfer::receive_number),
                )
                .branch(
                    case![DeleteTransferState::Confirm(number)]
                        .endpoint(delete_transfer::receive_confirm_text),
                ),
        )
        .branch(
            case![SetLanguage(state)]
                .branch(case![SetLanguageState::AskLangid].endpoint(set_language::receive_langid)),
        )
        .branch(
            case![SetCurrency(state)].branch(
                case![SetCurrencyState::AskCurrency].endpoint(set_currency::receive_currency),
            ),
        )
        .branch(case![ListExpenses(state)].branch(
            case![ListExpensesState::AskDescription].endpoint(list_expenses::receive_description),
        ))
        .branch(
            case![Transfer(state)]
                .branch(case![TransferState::AskFrom].endpoint(transfer::receive_from_text))
                .branch(case![TransferState::AskTo(from)].endpoint(transfer::receive_to_text))
                .branch(
                    case![TransferState::AskAmount(from, to)].endpoint(transfer::receive_amount),
                ),
        )
        .branch(case![ClearTravelers(state)].branch(
            case![ClearTravelersState::Confirm].endpoint(clear_travelers::receive_confirm_text),
        ))
        .branch(case![ClearExpenses(state)].branch(
            case![ClearExpensesState::Confirm].endpoint(clear_expenses::receive_confirm_text),
        ))
        .branch(case![ClearTransfers(state)].branch(
            case![ClearTransfersState::Confirm].endpoint(clear_transfers::receive_confirm_text),
        ))
        .branch(
            case![ClearAll(state)]
                .branch(case![ClearAllState::Confirm].endpoint(clear_all::receive_confirm_text)),
        )
}

/// All callback-data prefixes used by pending-command dialogue keyboards.
const CALLBACK_PREFIXES: &[&str] = &[
    set_language::CALLBACK_PREFIX,
    set_currency::CALLBACK_PREFIX,
    delete_traveler::CALLBACK_PREFIX,
    transfer::CALLBACK_PREFIX_FROM,
    transfer::CALLBACK_PREFIX_TO,
    delete_expense::CALLBACK_PREFIX,
    show_expense::CALLBACK_PREFIX,
    delete_transfer::CALLBACK_PREFIX,
    clear_travelers::CALLBACK_PREFIX,
    clear_expenses::CALLBACK_PREFIX,
    clear_transfers::CALLBACK_PREFIX,
    clear_all::CALLBACK_PREFIX,
];

/// Returns `true` if the callback data matches any pending-command dialogue
/// prefix.
pub fn is_pending_callback(data: &str) -> bool {
    CALLBACK_PREFIXES.iter().any(|p| data.starts_with(p))
}

/// Returns the dispatcher subtree that handles inline-keyboard callbacks for
/// pending-command dialogues. Composed into [`crate::handler_tree`] alongside
/// other callback branches.
pub fn callback_branch() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use PendingCommandState::*;
    use teloxide::dptree::{self, case};

    dptree::entry()
        .enter_dialogue::<CallbackQuery, PendingCommandStorage, PendingCommandState>()
        .branch(
            case![SetLanguage(state)].branch(
                case![SetLanguageState::AskLangid].endpoint(set_language::receive_callback),
            ),
        )
        .branch(
            case![SetCurrency(state)].branch(
                case![SetCurrencyState::AskCurrency].endpoint(set_currency::receive_callback),
            ),
        )
        .branch(
            case![DeleteTraveler(state)]
                .branch(
                    case![DeleteTravelerState::AskName].endpoint(delete_traveler::receive_callback),
                )
                .branch(
                    case![DeleteTravelerState::Confirm(name)]
                        .endpoint(delete_traveler::receive_confirm_callback),
                ),
        )
        .branch(
            case![Transfer(state)]
                .branch(case![TransferState::AskFrom].endpoint(transfer::receive_from_callback))
                .branch(case![TransferState::AskTo(from)].endpoint(transfer::receive_to_callback)),
        )
        .branch(
            case![DeleteExpense(state)]
                .branch(
                    case![DeleteExpenseState::AskNumber].endpoint(delete_expense::receive_callback),
                )
                .branch(
                    case![DeleteExpenseState::Confirm(number)]
                        .endpoint(delete_expense::receive_confirm_callback),
                ),
        )
        .branch(
            case![ShowExpense(state)].branch(
                case![ShowExpenseState::AskNumber].endpoint(show_expense::receive_callback),
            ),
        )
        .branch(
            case![DeleteTransfer(state)]
                .branch(
                    case![DeleteTransferState::AskNumber]
                        .endpoint(delete_transfer::receive_callback),
                )
                .branch(
                    case![DeleteTransferState::Confirm(number)]
                        .endpoint(delete_transfer::receive_confirm_callback),
                ),
        )
        .branch(case![ClearTravelers(state)].branch(
            case![ClearTravelersState::Confirm].endpoint(clear_travelers::receive_confirm_callback),
        ))
        .branch(case![ClearExpenses(state)].branch(
            case![ClearExpensesState::Confirm].endpoint(clear_expenses::receive_confirm_callback),
        ))
        .branch(case![ClearTransfers(state)].branch(
            case![ClearTransfersState::Confirm].endpoint(clear_transfers::receive_confirm_callback),
        ))
        .branch(
            case![ClearAll(state)].branch(
                case![ClearAllState::Confirm].endpoint(clear_all::receive_confirm_callback),
            ),
        )
}
