//! `/transfer` dialogue: guides the user through selecting a sender,
//! receiver, and amount when the command is invoked without inline arguments.
//! Uses traveler-picker keyboards for the "from" and "to" steps; free-text
//! input is accepted as a fallback at each step.

use crate::{
    Context, HandlerResult,
    commands::transfer as cmd_transfer,
    consts::{LOG_DEBUG_START, LOG_DEBUG_SUCCESS},
    dialogues::pending_command_dialogue::{PendingCommandDialogue, PendingCommandState},
    i18n::{self, Translate, TranslateWithArgs},
    keyboard::{self, DEFAULT_ROWS_PER_PAGE, PaginatedKeyboardConfig, PickerItem},
    traveler::{Name, Traveler},
};
use macro_rules_attribute::apply;
use rust_decimal::Decimal;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, Message},
};
use tracing::Level;

// ─── Callback constants ──────────────────────────────────────────────────────

/// Prefix for the "from" traveler picker.
pub const CALLBACK_PREFIX_FROM: &str = "xfrfrom:";
/// Prefix for the "to" traveler picker.
pub const CALLBACK_PREFIX_TO: &str = "xfrto:";

/// Cancel callback for the "from" step.
pub const CANCEL_CALLBACK_FROM: &str = "xfrfrom:__cancel__";
/// Noop callback for the "from" step.
const NOOP_CALLBACK_FROM: &str = "xfrfrom:__noop__";

/// Cancel callback for the "to" step.
pub const CANCEL_CALLBACK_TO: &str = "xfrto:__cancel__";
/// Noop callback for the "to" step.
const NOOP_CALLBACK_TO: &str = "xfrto:__noop__";

// ─── State ───────────────────────────────────────────────────────────────────

/// Newtype wrapper so dptree can distinguish the "from" name from other `Name`
/// values in the dependency container.
#[derive(Debug, Clone)]
pub struct TransferFrom(pub Name);

/// Newtype wrapper for the "to" name.
#[derive(Debug, Clone)]
pub struct TransferTo(pub Name);

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum TransferState {
    AskFrom,
    AskTo(TransferFrom),
    AskAmount(TransferFrom, TransferTo),
}

// ─── Start (shows "from" keyboard) ──────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn start(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let prompt = i18n::dialogues::TRANSFER_ASK_FROM.translate(ctx.clone());
    let mut request = bot.send_message(msg.chat.id, prompt);
    if let Some(kb) = keyboard::travelers_keyboard(keyboard::TravelersKeyboardConfig {
        db,
        chat_id: msg.chat.id,
        prefix: CALLBACK_PREFIX_FROM,
        cancel_callback: CANCEL_CALLBACK_FROM,
        noop_callback: NOOP_CALLBACK_FROM,
        show_cancel: true,
        ctx,
    })
    .await
    {
        request = request.reply_markup(kb);
    }
    request.await?;
    dialogue
        .update(PendingCommandState::Transfer(TransferState::AskFrom))
        .await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

/// Entry point when `/transfer <from>` was invoked with just the sender name.
/// Skips the AskFrom step and goes directly to AskTo.
#[apply(trace_state_db)]
pub async fn start_with_from(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    args: String,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let from = match Name::from_str(args.trim()) {
        Ok(n) => n,
        Err(_) => {
            // Invalid from name — fall back to full dialogue starting from AskFrom.
            let prompt = i18n::dialogues::TRANSFER_ASK_FROM.translate(ctx.clone());
            let mut request = bot.send_message(msg.chat.id, prompt);
            if let Some(kb) = keyboard::travelers_keyboard(keyboard::TravelersKeyboardConfig {
                db,
                chat_id: msg.chat.id,
                prefix: CALLBACK_PREFIX_FROM,
                cancel_callback: CANCEL_CALLBACK_FROM,
                noop_callback: NOOP_CALLBACK_FROM,
                show_cancel: true,
                ctx,
            })
            .await
            {
                request = request.reply_markup(kb);
            }
            request.await?;
            dialogue
                .update(PendingCommandState::Transfer(TransferState::AskFrom))
                .await?;
            return Ok(());
        }
    };
    // Check that the traveler exists in the DB.
    if Traveler::db_select_by_name(db.clone(), msg.chat.id, &from)
        .await?
        .is_none()
    {
        reprompt_from_not_found(db, &bot, msg.chat.id, &from, ctx.clone()).await?;
        dialogue
            .update(PendingCommandState::Transfer(TransferState::AskFrom))
            .await?;
        return Ok(());
    }
    transition_to_ask_to(db, &bot, &dialogue, msg.chat.id, from, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

/// Entry point when `/transfer <from> <to>` was invoked with sender and
/// receiver. Skips AskFrom and AskTo, goes directly to AskAmount.
#[apply(trace_state_db)]
pub async fn start_with_from_to(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    args: String,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");
    let mut parts = args.split_whitespace();
    let from_str = parts.next().unwrap_or("");
    let to_str = parts.next().unwrap_or("");
    let from = match Name::from_str(from_str) {
        Ok(n) => n,
        Err(_) => {
            // Invalid from — start full dialogue.
            let prompt = i18n::dialogues::TRANSFER_ASK_FROM.translate(ctx.clone());
            let mut request = bot.send_message(msg.chat.id, prompt);
            if let Some(kb) = keyboard::travelers_keyboard(keyboard::TravelersKeyboardConfig {
                db,
                chat_id: msg.chat.id,
                prefix: CALLBACK_PREFIX_FROM,
                cancel_callback: CANCEL_CALLBACK_FROM,
                noop_callback: NOOP_CALLBACK_FROM,
                show_cancel: true,
                ctx,
            })
            .await
            {
                request = request.reply_markup(kb);
            }
            request.await?;
            dialogue
                .update(PendingCommandState::Transfer(TransferState::AskFrom))
                .await?;
            return Ok(());
        }
    };
    // Check that the "from" traveler exists in the DB.
    if Traveler::db_select_by_name(db.clone(), msg.chat.id, &from)
        .await?
        .is_none()
    {
        reprompt_from_not_found(db, &bot, msg.chat.id, &from, ctx.clone()).await?;
        dialogue
            .update(PendingCommandState::Transfer(TransferState::AskFrom))
            .await?;
        return Ok(());
    }
    let to = match Name::from_str(to_str) {
        Ok(n) => n,
        Err(_) => {
            // Invalid to — start from AskTo with the valid from.
            transition_to_ask_to(db, &bot, &dialogue, msg.chat.id, from, ctx).await?;
            return Ok(());
        }
    };
    // Check that the "to" traveler exists in the DB.
    if Traveler::db_select_by_name(db.clone(), msg.chat.id, &to)
        .await?
        .is_none()
    {
        reprompt_to_not_found(db, &bot, msg.chat.id, &from, &to, ctx.clone()).await?;
        dialogue
            .update(PendingCommandState::Transfer(TransferState::AskTo(
                TransferFrom(from),
            )))
            .await?;
        return Ok(());
    }
    let from_wrapper = TransferFrom(from);
    transition_to_ask_amount(&bot, &dialogue, msg.chat.id, from_wrapper, to, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── "From" text handler ─────────────────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn receive_from_text(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    let name = match parse_name(text) {
        Ok(n) => n,
        Err(_) => {
            reprompt_ask_from(db, &bot, msg.chat.id, ctx).await?;
            return Ok(());
        }
    };

    // Check that the traveler exists in the DB.
    if Traveler::db_select_by_name(db.clone(), msg.chat.id, &name)
        .await?
        .is_none()
    {
        reprompt_from_not_found(db, &bot, msg.chat.id, &name, ctx).await?;
        return Ok(());
    }

    transition_to_ask_to(db, &bot, &dialogue, msg.chat.id, name, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── "From" callback handler ─────────────────────────────────────────────────

#[apply(trace_callback)]
pub async fn receive_from_callback(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let action = keyboard::handle_callback_prelude(
        &bot,
        &dialogue,
        &q,
        &ctx,
        &keyboard::CallbackConfig {
            cancel_callback: CANCEL_CALLBACK_FROM,
            noop_callback: NOOP_CALLBACK_FROM,
            prefix: CALLBACK_PREFIX_FROM,
            running_process_key: i18n::commands::RUNNING_PROCESS_TRANSFER,
        },
    )
    .await?;

    let keyboard::CallbackAction::Selection { value: raw, msg } = action else {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    };

    let Ok(name) = Name::from_str(&raw) else {
        tracing::warn!("Invalid name in callback data: {raw:?}");
        return Ok(());
    };

    // Remove the inline keyboard from the "from" prompt.
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    transition_to_ask_to(db, &bot, &dialogue, msg.chat.id, name, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── "To" text handler ───────────────────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn receive_to_text(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    from: TransferFrom,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    let name = match parse_name(text) {
        Ok(n) => n,
        Err(_) => {
            let prompt = i18n::dialogues::TRANSFER_ASK_TO_REPROMPT.translate(ctx.clone());
            let mut request = bot.send_message(msg.chat.id, prompt);
            if let Some(kb) = travelers_keyboard_excluding(db, msg.chat.id, &from.0, ctx).await {
                request = request.reply_markup(kb);
            }
            request.await?;
            return Ok(());
        }
    };

    // Check that the traveler exists in the DB.
    if Traveler::db_select_by_name(db.clone(), msg.chat.id, &name)
        .await?
        .is_none()
    {
        reprompt_to_not_found(db, &bot, msg.chat.id, &from.0, &name, ctx).await?;
        return Ok(());
    }

    transition_to_ask_amount(&bot, &dialogue, msg.chat.id, from, name, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── "To" callback handler ───────────────────────────────────────────────────

#[apply(trace_callback)]
pub async fn receive_to_callback(
    _db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    q: CallbackQuery,
    from: TransferFrom,
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let action = keyboard::handle_callback_prelude(
        &bot,
        &dialogue,
        &q,
        &ctx,
        &keyboard::CallbackConfig {
            cancel_callback: CANCEL_CALLBACK_TO,
            noop_callback: NOOP_CALLBACK_TO,
            prefix: CALLBACK_PREFIX_TO,
            running_process_key: i18n::commands::RUNNING_PROCESS_TRANSFER,
        },
    )
    .await?;

    let keyboard::CallbackAction::Selection { value: raw, msg } = action else {
        tracing::debug!("{LOG_DEBUG_SUCCESS}");
        return Ok(());
    };

    let Ok(name) = Name::from_str(&raw) else {
        tracing::warn!("Invalid name in callback data: {raw:?}");
        return Ok(());
    };

    // Remove the inline keyboard from the "to" prompt.
    let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;

    transition_to_ask_amount(&bot, &dialogue, msg.chat.id, from, name, ctx).await?;
    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── Amount text handler ─────────────────────────────────────────────────────

#[apply(trace_state_db)]
pub async fn receive_amount(
    db: Arc<Surreal<Any>>,
    bot: Bot,
    dialogue: PendingCommandDialogue,
    msg: Message,
    (from, to): (TransferFrom, TransferTo),
    ctx: Arc<Mutex<Context>>,
) -> HandlerResult {
    tracing::debug!("{LOG_DEBUG_START}");

    let text = msg.text().map(str::trim).unwrap_or("");
    let amount = match Decimal::from_str(text) {
        Ok(d) => d,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                i18n::dialogues::TRANSFER_INVALID_AMOUNT.translate(ctx),
            )
            .await?;
            return Ok(());
        }
    };

    let result = cmd_transfer(db.clone(), &msg, from.0, to.0, amount, ctx.clone()).await;
    match result {
        Ok(reply) => {
            bot.send_message(msg.chat.id, reply).await?;
            dialogue.exit().await?;
        }
        Err(err) => {
            use crate::i18n::Translate as _;
            let reply = err.translate(ctx.clone());
            bot.send_message(msg.chat.id, reply).await?;
            // Stay in AskAmount — user can retry with a different amount.
        }
    }

    tracing::debug!("{LOG_DEBUG_SUCCESS}");
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Number of traveler buttons per row.
const TRAVELERS_PER_ROW: usize = 2;

fn parse_name(text: &str) -> Result<Name, ()> {
    if text.is_empty() {
        return Err(());
    }
    Name::from_str(text).map_err(|_| ())
}

/// Re-sends the "from" prompt (with keyboard) after an invalid name.
async fn reprompt_ask_from(
    db: Arc<Surreal<Any>>,
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prompt = i18n::dialogues::TRANSFER_ASK_FROM_REPROMPT.translate(ctx.clone());
    let mut request = bot.send_message(chat_id, prompt);
    if let Some(kb) = keyboard::travelers_keyboard(keyboard::TravelersKeyboardConfig {
        db,
        chat_id,
        prefix: CALLBACK_PREFIX_FROM,
        cancel_callback: CANCEL_CALLBACK_FROM,
        noop_callback: NOOP_CALLBACK_FROM,
        show_cancel: true,
        ctx,
    })
    .await
    {
        request = request.reply_markup(kb);
    }
    request.await?;
    Ok(())
}

/// Re-sends the "from" prompt (with keyboard) when the traveler is not found.
async fn reprompt_from_not_found(
    db: Arc<Surreal<Any>>,
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    name: &Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prompt = i18n::dialogues::TRANSFER_FROM_NOT_FOUND.translate_with_args(
        ctx.clone(),
        &maplit::hashmap! { i18n::args::NAME.into() => name.to_string().into() },
    );
    let mut request = bot.send_message(chat_id, prompt);
    if let Some(kb) = keyboard::travelers_keyboard(keyboard::TravelersKeyboardConfig {
        db,
        chat_id,
        prefix: CALLBACK_PREFIX_FROM,
        cancel_callback: CANCEL_CALLBACK_FROM,
        noop_callback: NOOP_CALLBACK_FROM,
        show_cancel: true,
        ctx,
    })
    .await
    {
        request = request.reply_markup(kb);
    }
    request.await?;
    Ok(())
}

/// Re-sends the "to" prompt (with keyboard) when the traveler is not found.
async fn reprompt_to_not_found(
    db: Arc<Surreal<Any>>,
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    exclude: &Name,
    name: &Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prompt = i18n::dialogues::TRANSFER_TO_NOT_FOUND.translate_with_args(
        ctx.clone(),
        &maplit::hashmap! { i18n::args::NAME.into() => name.to_string().into() },
    );
    let mut request = bot.send_message(chat_id, prompt);
    if let Some(kb) = travelers_keyboard_excluding(db, chat_id, exclude, ctx).await {
        request = request.reply_markup(kb);
    }
    request.await?;
    Ok(())
}

/// Transition from AskFrom to AskTo: show the "to" keyboard (excluding the
/// chosen "from" traveler) and update dialogue state.
async fn transition_to_ask_to(
    db: Arc<Surreal<Any>>,
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    from: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prompt = i18n::dialogues::TRANSFER_ASK_TO.translate_with_args(
        ctx.clone(),
        &maplit::hashmap! { i18n::args::NAME.into() => from.to_string().into() },
    );
    let mut request = bot.send_message(chat_id, prompt);

    // Build keyboard excluding the "from" traveler.
    if let Some(kb) = travelers_keyboard_excluding(db, chat_id, &from, ctx).await {
        request = request.reply_markup(kb);
    }
    request.await?;

    dialogue
        .update(PendingCommandState::Transfer(TransferState::AskTo(
            TransferFrom(from),
        )))
        .await?;
    Ok(())
}

/// Transition from AskTo to AskAmount: prompt for the amount.
async fn transition_to_ask_amount(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    chat_id: teloxide::types::ChatId,
    from: TransferFrom,
    to: Name,
    ctx: Arc<Mutex<Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prompt = i18n::dialogues::TRANSFER_ASK_AMOUNT.translate_with_args(
        ctx,
        &maplit::hashmap! { i18n::args::NAME.into() => from.0.to_string().into() },
    );
    bot.send_message(chat_id, prompt).await?;
    dialogue
        .update(PendingCommandState::Transfer(TransferState::AskAmount(
            from,
            TransferTo(to),
        )))
        .await?;
    Ok(())
}

/// Builds a traveler-picker keyboard excluding one specific name.
async fn travelers_keyboard_excluding(
    db: Arc<Surreal<Any>>,
    chat_id: teloxide::types::ChatId,
    exclude: &Name,
    ctx: Arc<Mutex<Context>>,
) -> Option<teloxide::types::InlineKeyboardMarkup> {
    let travelers = Traveler::db_select(db, chat_id).await.ok()?;
    let items: Vec<PickerItem> = travelers
        .into_iter()
        .filter(|t| t.name.to_lowercase() != exclude.to_lowercase())
        .map(|t| {
            let name = t.name.to_string();
            PickerItem {
                label: name.clone(),
                value: name,
            }
        })
        .collect();
    if items.is_empty() {
        return None;
    }
    keyboard::paginated_keyboard(PaginatedKeyboardConfig {
        items: &items,
        page: 0,
        columns: TRAVELERS_PER_ROW,
        rows_per_page: DEFAULT_ROWS_PER_PAGE,
        prefix: CALLBACK_PREFIX_TO,
        cancel_callback: CANCEL_CALLBACK_TO,
        noop_callback: NOOP_CALLBACK_TO,
        action_buttons: &[],
        show_cancel: true,
        ctx,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        db::db,
        i18n::{self, Translate, TranslateWithArgs},
        tests::{TestBot, helpers},
    };
    use maplit::hashmap;

    // ─── Full dialogue flow ──────────────────────────────────────────────

    test! { full_dialogue_from_to_amount,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Start dialogue
        bot.update("/transfer");
        let ask_from = i18n::dialogues::TRANSFER_ASK_FROM.translate_default();
        bot.test_last_message(&ask_from).await;

        // Enter "from"
        bot.update("Alice");
        let ask_to = i18n::dialogues::TRANSFER_ASK_TO.translate_with_args_default(&hashmap! {
            i18n::args::NAME.into() => "Alice".into(),
        });
        bot.test_last_message(&ask_to).await;

        // Enter "to"
        bot.update("Bob");
        let ask_amount = i18n::dialogues::TRANSFER_ASK_AMOUNT.translate_with_args_default(&hashmap! {
            i18n::args::NAME.into() => "Alice".into(),
        });
        bot.test_last_message(&ask_amount).await;

        // Enter amount
        bot.update("50");
        let response = i18n::commands::TRANSFER_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    // ─── Invalid "from" reprompts ────────────────────────────────────────

    test! { invalid_from_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Start dialogue
        bot.update("/transfer");
        bot.dispatch().await;

        // Enter invalid name (starts with slash)
        bot.update("/invalid");
        let reprompt = i18n::dialogues::TRANSFER_ASK_FROM_REPROMPT.translate_default();
        bot.test_last_message(&reprompt).await;

        // Now enter valid name
        bot.update("Alice");
        let ask_to = i18n::dialogues::TRANSFER_ASK_TO.translate_with_args_default(&hashmap! {
            i18n::args::NAME.into() => "Alice".into(),
        });
        bot.test_last_message(&ask_to).await;
    }

    // ─── Invalid "to" reprompts ──────────────────────────────────────────

    test! { invalid_to_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Start dialogue and select "from"
        bot.update("/transfer");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;

        // Enter invalid "to" name
        bot.update("/invalid");
        let reprompt = i18n::dialogues::TRANSFER_ASK_TO_REPROMPT.translate_default();
        bot.test_last_message(&reprompt).await;

        // Now enter valid name
        bot.update("Bob");
        let ask_amount = i18n::dialogues::TRANSFER_ASK_AMOUNT.translate_with_args_default(&hashmap! {
            i18n::args::NAME.into() => "Alice".into(),
        });
        bot.test_last_message(&ask_amount).await;
    }

    // ─── Invalid amount reprompts ────────────────────────────────────────

    test! { invalid_amount_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        // Navigate to AskAmount
        bot.update("/transfer");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;
        bot.update("Bob");
        bot.dispatch().await;

        // Enter non-numeric amount
        bot.update("not a number");
        let reprompt = i18n::dialogues::TRANSFER_INVALID_AMOUNT.translate_default();
        bot.test_last_message(&reprompt).await;

        // Enter valid amount
        bot.update("100");
        let response = i18n::commands::TRANSFER_OK.translate_default();
        bot.test_last_message(&response).await;
    }

    // ─── Cancel during dialogue ──────────────────────────────────────────

    test! { cancel_at_ask_from,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        bot.update("/transfer");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = helpers::cancel_ok_for(i18n::commands::RUNNING_PROCESS_TRANSFER);
        bot.test_last_message(&response).await;
    }

    test! { cancel_at_ask_to,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/transfer");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = helpers::cancel_ok_for(i18n::commands::RUNNING_PROCESS_TRANSFER);
        bot.test_last_message(&response).await;
    }

    test! { cancel_at_ask_amount,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;
        helpers::add_traveler(&mut bot, "Bob").await;

        bot.update("/transfer");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;
        bot.update("Bob");
        bot.dispatch().await;

        bot.update("/cancel");
        let response = helpers::cancel_ok_for(i18n::commands::RUNNING_PROCESS_TRANSFER);
        bot.test_last_message(&response).await;
    }

    // ─── Sender not found (reprompts at AskFrom) ─────────────────────────

    test! { sender_not_found_reprompts_at_ask_from,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Bob").await;

        bot.update("/transfer");
        bot.dispatch().await;

        // "Alice" doesn't exist → reprompt
        bot.update("Alice");
        let reprompt = i18n::dialogues::TRANSFER_FROM_NOT_FOUND.translate_with_args_default(
            &hashmap! { i18n::args::NAME.into() => "Alice".into() },
        );
        bot.test_last_message(&reprompt).await;

        // Now enter valid traveler
        bot.update("Bob");
        let ask_to = i18n::dialogues::TRANSFER_ASK_TO.translate_with_args_default(&hashmap! {
            i18n::args::NAME.into() => "Bob".into(),
        });
        bot.test_last_message(&ask_to).await;
    }

    // ─── Receiver not found (reprompts at AskTo) ─────────────────────────

    test! { receiver_not_found_reprompts_at_ask_to,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/transfer");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;

        // "Bob" doesn't exist → reprompt
        bot.update("Bob");
        let reprompt = i18n::dialogues::TRANSFER_TO_NOT_FOUND.translate_with_args_default(
            &hashmap! { i18n::args::NAME.into() => "Bob".into() },
        );
        bot.test_last_message(&reprompt).await;
    }

    // ─── Same sender/receiver ────────────────────────────────────────────

    test! { same_sender_receiver_in_dialogue,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/transfer");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;
        bot.update("Alice");
        bot.dispatch().await;

        bot.update("100");
        let response = i18n::commands::TRANSFER_SAME_SENDER_RECEIVER.translate_with_args_default(
            &hashmap! { i18n::args::NAME.into() => "Alice".into() },
        );
        bot.test_last_message(&response).await;
    }

    // ─── Partial command: invalid from falls back ────────────────────────

    test! { partial_command_invalid_from_falls_back_to_ask_from,
        let db = db().await;
        let mut bot = TestBot::new(db, "/transfer /bad");

        // Invalid from name -> should fall back to AskFrom prompt
        let ask_from = i18n::dialogues::TRANSFER_ASK_FROM.translate_default();
        bot.test_last_message(&ask_from).await;
    }

    // ─── Partial command: valid from, invalid to falls back ──────────────

    test! { partial_command_valid_from_invalid_to_falls_back_to_ask_to,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/transfer Alice /bad");

        // Valid from, invalid to -> should fall back to AskTo
        let ask_to = i18n::dialogues::TRANSFER_ASK_TO.translate_with_args_default(&hashmap! {
            i18n::args::NAME.into() => "Alice".into(),
        });
        bot.test_last_message(&ask_to).await;
    }

    // ─── Partial command: from not found ─────────────────────────────────

    test! { partial_command_from_not_found_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "/transfer Zara");

        // "Zara" parses as a valid Name but doesn't exist → not-found reprompt
        let reprompt = i18n::dialogues::TRANSFER_FROM_NOT_FOUND.translate_with_args_default(
            &hashmap! { i18n::args::NAME.into() => "Zara".into() },
        );
        bot.test_last_message(&reprompt).await;
    }

    // ─── Partial command: from exists, to not found ──────────────────────

    test! { partial_command_to_not_found_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        helpers::add_traveler(&mut bot, "Alice").await;

        bot.update("/transfer Alice Zara");

        // "Alice" exists, "Zara" doesn't → not-found reprompt at AskTo
        let reprompt = i18n::dialogues::TRANSFER_TO_NOT_FOUND.translate_with_args_default(
            &hashmap! { i18n::args::NAME.into() => "Zara".into() },
        );
        bot.test_last_message(&reprompt).await;
    }

    // ─── Empty from text is reprompted ───────────────────────────────────

    test! { empty_from_text_reprompts,
        let db = db().await;
        let mut bot = TestBot::new(db, "");

        bot.update("/transfer");
        bot.dispatch().await;

        // Send empty text (whitespace only)
        bot.update("   ");
        let reprompt = i18n::dialogues::TRANSFER_ASK_FROM_REPROMPT.translate_default();
        bot.test_last_message(&reprompt).await;
    }
}
