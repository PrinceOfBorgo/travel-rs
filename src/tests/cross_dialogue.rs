//! Cross-dialogue interaction tests.
//!
//! Each dialogue must be mutually exclusive with every other dialogue for a
//! given chat: starting one while another is in progress must be refused
//! with `process-already-running`, and the original dialogue must remain
//! alive and able to make progress.
//!
//! When you add a new dialogue, add one entry per existing dialogue to the
//! collision matrix in `mod collisions` below. The shared
//! [`assert_dialogues_are_mutually_exclusive`] helper keeps each entry to a
//! single call.

use crate::{
    db::db,
    i18n::{Translate, TranslateWithArgs},
    tests::TestBot,
};
use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};

/// Verifies that starting `intruder_command` while `host_command`'s dialogue
/// is mid-flow is refused with `process-already-running`, and that the host
/// dialogue is still alive afterwards: feeding `host_followup` to it
/// produces `host_next_prompt`.
///
/// The helper makes each collision case a one-liner so the matrix scales
/// O(n²) without test boilerplate growing with it.
async fn assert_dialogues_are_mutually_exclusive(
    db: Arc<Surreal<Any>>,
    host_command: &str,
    intruder_command: &str,
    host_followup: &str,
    host_next_prompt: &str,
) {
    let mut bot = TestBot::new(db, host_command);
    bot.dispatch().await;

    // Intruder command is refused.
    bot.update(intruder_command);
    let process = host_command.trim_start_matches('/');
    let refused = crate::i18n::commands::PROCESS_ALREADY_RUNNING.translate_with_args_default(
        &maplit::hashmap! {
            crate::i18n::args::PROCESS.into() => format!("/{process}").into(),
        },
    );
    bot.test_last_message(&refused).await;

    // Host dialogue is still alive and progresses on the next input.
    bot.update(host_followup);
    bot.test_last_message(host_next_prompt).await;
}

mod collisions {
    use super::*;
    use crate::i18n;

    // ------------------------------------------------------------------
    // AddExpense  <->  AddTraveler (dialogue form)
    // ------------------------------------------------------------------

    test! { add_traveler_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db,
            "/addexpense",
            "/addtraveler",
            host_followup,
            &host_next_prompt,
        ).await;
    }

    test! { add_expense_blocked_while_add_traveler_dialogue_running,
        let db = db().await;
        // Completing the AddTraveler dialogue with "Alice" yields ADD_TRAVELER_OK.
        let host_followup = "Alice";
        let host_next_prompt = i18n::commands::ADD_TRAVELER_OK.translate_with_args_default(
            &maplit::hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        assert_dialogues_are_mutually_exclusive(
            db,
            "/addtraveler",
            "/addexpense",
            host_followup,
            &host_next_prompt,
        ).await;
    }

    // ------------------------------------------------------------------
    // AddExpense  <->  DeleteTraveler (dialogue form)
    // ------------------------------------------------------------------

    test! { delete_traveler_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db,
            "/addexpense",
            "/deletetraveler",
            host_followup,
            &host_next_prompt,
        ).await;
    }

    test! { add_expense_blocked_while_delete_traveler_dialogue_running,
        let db = db().await;
        let host_followup = "Alice";
        // After the not-found reply, the dialogue re-prompts the user.
        let host_next_prompt = i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db,
            "/deletetraveler",
            "/addexpense",
            host_followup,
            &host_next_prompt,
        ).await;
    }

    // ------------------------------------------------------------------
    // AddTraveler  <->  DeleteTraveler (both dialogue form)
    // ------------------------------------------------------------------

    test! { delete_traveler_dialogue_blocked_while_add_traveler_dialogue_running,
        let db = db().await;
        let host_followup = "Alice";
        let host_next_prompt = i18n::commands::ADD_TRAVELER_OK.translate_with_args_default(
            &maplit::hashmap! {i18n::args::NAME.into() => "Alice".into()},
        );
        assert_dialogues_are_mutually_exclusive(
            db,
            "/addtraveler",
            "/deletetraveler",
            host_followup,
            &host_next_prompt,
        ).await;
    }

    test! { add_traveler_dialogue_blocked_while_delete_traveler_dialogue_running,
        let db = db().await;
        let host_followup = "Alice";
        // After the not-found reply, the dialogue re-prompts the user.
        let host_next_prompt = i18n::dialogues::DELETE_TRAVELER_ASK_NAME.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db,
            "/deletetraveler",
            "/addtraveler",
            host_followup,
            &host_next_prompt,
        ).await;
    }

    // ------------------------------------------------------------------
    // The remaining pending-command dialogues. Each new dialogue is tested
    // against AddExpense (the multi-step dialogue) in both directions, and
    // a representative collision against AddTraveler.
    // ------------------------------------------------------------------

    test! { delete_expense_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/addexpense", "/deleteexpense", host_followup, &host_next_prompt,
        ).await;
    }

    test! { add_expense_blocked_while_delete_expense_dialogue_running,
        let db = db().await;
        let host_followup = "999";
        // After the not-found reply, the dialogue re-prompts the user.
        let host_next_prompt = i18n::dialogues::DELETE_EXPENSE_ASK_NUMBER.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/deleteexpense", "/addexpense", host_followup, &host_next_prompt,
        ).await;
    }

    test! { show_expense_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/addexpense", "/showexpense", host_followup, &host_next_prompt,
        ).await;
    }

    test! { add_expense_blocked_while_show_expense_dialogue_running,
        let db = db().await;
        let host_followup = "999";
        // After the not-found reply, the dialogue re-prompts the user.
        let host_next_prompt = i18n::dialogues::SHOW_EXPENSE_ASK_NUMBER.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/showexpense", "/addexpense", host_followup, &host_next_prompt,
        ).await;
    }

    test! { delete_transfer_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/addexpense", "/deletetransfer", host_followup, &host_next_prompt,
        ).await;
    }

    test! { add_expense_blocked_while_delete_transfer_dialogue_running,
        let db = db().await;
        let host_followup = "999";
        // After the not-found reply, the dialogue re-prompts the user.
        let host_next_prompt = i18n::dialogues::DELETE_TRANSFER_ASK_NUMBER.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/deletetransfer", "/addexpense", host_followup, &host_next_prompt,
        ).await;
    }

    test! { set_language_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/addexpense", "/setlanguage", host_followup, &host_next_prompt,
        ).await;
    }

    test! { set_currency_dialogue_blocked_while_add_expense_running,
        let db = db().await;
        let host_followup = "My expense";
        let host_next_prompt = i18n::dialogues::ADD_EXPENSE_ASK_AMOUNT.translate_default();
        assert_dialogues_are_mutually_exclusive(
            db, "/addexpense", "/setcurrency", host_followup, &host_next_prompt,
        ).await;
    }

    // ------------------------------------------------------------------
    // ListExpenses (callback-initiated)  <->  AddExpense
    // Since list_expenses dialogue is started from a callback (not a
    // command), we seed the state manually and verify mutual exclusion.
    // ------------------------------------------------------------------

    test! { add_expense_blocked_while_list_expenses_dialogue_running,
        let db = db().await;
        let mut bot = TestBot::new(db, "/addtraveler Alice");
        bot.dispatch().await;

        // Seed the ListExpenses dialogue as if the "Filter…" button was tapped.
        use crate::dialogues::pending_command_dialogue::PendingCommandState;
        use crate::dialogues::pending_command_dialogue::list_expenses::ListExpensesState;
        use teloxide::dispatching::dialogue::Storage;

        let storage = bot.pending_command_storage();
        storage
            .update_dialogue(
                bot.chat_id(),
                PendingCommandState::ListExpenses(ListExpensesState::AskDescription),
            )
            .await
            .unwrap();

        // Attempting to start /addexpense should be refused.
        bot.update("/addexpense");
        let refused = i18n::commands::PROCESS_ALREADY_RUNNING.translate_with_args_default(
            &maplit::hashmap! {
                crate::i18n::args::PROCESS.into() =>
                    i18n::commands::RUNNING_PROCESS_LIST_EXPENSES.translate_default().into(),
            },
        );
        bot.test_last_message(&refused).await;

        // Host (list_expenses) dialogue is still alive: feeding a
        // description completes it.
        bot.update("Toll");
        bot.dispatch().await;

        // After completion, /cancel reports nothing running.
        bot.update("/cancel");
        let response = i18n::commands::CANCEL_NO_PROCESS_TO_CANCEL.translate_default();
        bot.test_last_message(&response).await;
    }
}
