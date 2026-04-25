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
    let refused = crate::i18n::commands::PROCESS_ALREADY_RUNNING.translate_default();
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
}
