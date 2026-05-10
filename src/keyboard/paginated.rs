//! Paginated picker keyboard for large item lists.
//!
//! This module provides the core paginated layout that other keyboard builders
//! (e.g. [`super::travelers_keyboard`]) compose on top of. The layout supports
//! configurable column count and optional action buttons.

use crate::{
    Context,
    consts::{BACK_LABEL, BLANK_LABEL, NEXT_LABEL},
    dialogues::pending_command_dialogue::PendingCommandDialogue,
    i18n::{self, Translate, TranslateWithArgs},
};
use std::sync::{Arc, Mutex};
use teloxide::Bot;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};
use teloxide::{requests::Requester, types::CallbackQuery};

use super::CallbackConfig;

/// Default number of item rows displayed per page.
pub const DEFAULT_ROWS_PER_PAGE: usize = 5;

/// Default number of columns (buttons per row).
pub const DEFAULT_COLUMNS: usize = 1;

/// Sentinel embedded in callback data for page navigation.
pub const PAGE_MARKER: &str = "__page__";

/// An item to display in a paginated picker keyboard.
pub struct PickerItem {
    /// The label shown on the inline button.
    pub label: String,
    /// The value embedded in the callback data (typically the item number).
    pub value: String,
}

/// Configuration for building a paginated keyboard.
pub struct PaginatedKeyboardConfig<'a> {
    /// Items to display (may span multiple pages).
    pub items: &'a [PickerItem],
    /// Zero-based page index.
    pub page: usize,
    /// Number of item buttons per row (1 = one per row, 2 = grid of two columns, etc.).
    /// Defaults to [`DEFAULT_COLUMNS`] when set to `0`.
    pub columns: usize,
    /// Maximum number of item rows shown per page. Defaults to
    /// [`DEFAULT_ROWS_PER_PAGE`] when set to `0`.
    pub rows_per_page: usize,
    /// Callback-data prefix prepended to each item value and to page markers.
    pub prefix: &'a str,
    /// Full cancel callback sentinel.
    pub cancel_callback: &'a str,
    /// Full noop callback sentinel (for spacers).
    pub noop_callback: &'a str,
    /// Optional extra action buttons placed in their own row between the
    /// navigation row and the cancel row.  Pass `&[]` when not needed.
    pub action_buttons: &'a [InlineKeyboardButton],
    /// Whether to show the cancel button row. Dialogue keyboards set this to
    /// `true`; stateless command keyboards set it to `false`.
    pub show_cancel: bool,
    /// Translation context.
    pub ctx: Arc<Mutex<Context>>,
}

/// Builds a paginated inline keyboard from a list of items.
///
/// - Items are laid out in rows of [`PaginatedKeyboardConfig::columns`] buttons.
/// - If there are more items than fit on one page (`rows_per_page * columns`),
///   `◀` / `▶` navigation buttons are shown.
/// - Optional action buttons are placed between nav and cancel.
/// - A cancel button is placed in the last row when `show_cancel` is `true`.
///
/// Returns `None` if `items` is empty.
pub fn paginated_keyboard(cfg: PaginatedKeyboardConfig<'_>) -> Option<InlineKeyboardMarkup> {
    let PaginatedKeyboardConfig {
        items,
        page,
        columns,
        rows_per_page,
        prefix,
        cancel_callback,
        noop_callback,
        action_buttons,
        show_cancel,
        ctx,
    } = cfg;

    if items.is_empty() {
        return None;
    }

    let columns = if columns == 0 {
        DEFAULT_COLUMNS
    } else {
        columns
    };
    let rows_per_page = if rows_per_page == 0 {
        DEFAULT_ROWS_PER_PAGE
    } else {
        rows_per_page
    };
    let items_per_page = rows_per_page * columns;
    let total_pages = items.len().div_ceil(items_per_page);
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * items_per_page;
    let end = (start + items_per_page).min(items.len());
    let page_items = &items[start..end];

    // Build item buttons, then chunk into rows of `columns`.
    let buttons: Vec<InlineKeyboardButton> = page_items
        .iter()
        .map(|item| {
            InlineKeyboardButton::callback(item.label.clone(), format!("{prefix}{}", item.value))
        })
        .collect();

    let mut rows: Vec<Vec<InlineKeyboardButton>> = if columns == 1 {
        // Fast path: one button per row (no spacers needed).
        buttons.into_iter().map(|b| vec![b]).collect()
    } else {
        // Grid layout with spacers on the last row if needed.
        let remainder = buttons.len() % columns;
        let blanks_needed = if remainder == 0 {
            0
        } else {
            columns - remainder
        };
        let mut all = buttons;
        all.extend(
            std::iter::repeat_with(|| {
                InlineKeyboardButton::callback(BLANK_LABEL.to_owned(), noop_callback.to_owned())
            })
            .take(blanks_needed),
        );
        all.chunks(columns)
            .map(<[InlineKeyboardButton]>::to_vec)
            .collect()
    };

    // Navigation row (only if more than one page).
    if total_pages > 1 {
        let mut nav_row = Vec::new();
        if page > 0 {
            nav_row.push(InlineKeyboardButton::callback(
                BACK_LABEL.to_owned(),
                format!("{prefix}{PAGE_MARKER}:{}", page - 1),
            ));
        } else {
            nav_row.push(InlineKeyboardButton::callback(
                BLANK_LABEL.to_owned(),
                noop_callback.to_owned(),
            ));
        }
        nav_row.push(InlineKeyboardButton::callback(
            format!("{}/{total_pages}", page + 1),
            noop_callback.to_owned(),
        ));
        if page + 1 < total_pages {
            nav_row.push(InlineKeyboardButton::callback(
                NEXT_LABEL.to_owned(),
                format!("{prefix}{PAGE_MARKER}:{}", page + 1),
            ));
        } else {
            nav_row.push(InlineKeyboardButton::callback(
                BLANK_LABEL.to_owned(),
                noop_callback.to_owned(),
            ));
        }
        rows.push(nav_row);
    }

    // Action buttons row (if any).
    if !action_buttons.is_empty() {
        rows.push(action_buttons.to_vec());
    }

    // Cancel row (only when requested).
    if show_cancel {
        rows.push(vec![InlineKeyboardButton::callback(
            i18n::labels::CANCEL_BUTTON.translate(ctx),
            cancel_callback.to_owned(),
        )]);
    }

    Some(InlineKeyboardMarkup::new(rows))
}

/// Result of handling a paginated callback via [`handle_paginated_callback`].
pub enum PaginatedCallbackAction {
    /// User selected an item; the value (stripped of prefix) is returned.
    Selection { value: String, msg: Box<Message> },
    /// User navigated to a different page; the caller should rebuild and
    /// edit the keyboard.
    PageChange { page: usize, msg: Box<Message> },
    /// Callback was fully handled (cancel, noop, inaccessible message,
    /// invalid callback data, or invalid page number).
    Handled,
}

/// Extended callback prelude for paginated keyboards. Handles cancel, noop,
/// page navigation, and item selection.
pub async fn handle_paginated_callback(
    bot: &Bot,
    dialogue: &PendingCommandDialogue,
    q: &CallbackQuery,
    ctx: &Arc<Mutex<Context>>,
    config: &CallbackConfig<'_>,
) -> Result<PaginatedCallbackAction, Box<dyn std::error::Error + Send + Sync>> {
    let _ = bot.answer_callback_query(q.id.clone()).await;

    let Some(msg) = q.regular_message().cloned() else {
        tracing::warn!("Callback query without an accessible message; ignoring");
        return Ok(PaginatedCallbackAction::Handled);
    };

    let data = q.data.as_deref().unwrap_or("");

    if data == config.noop_callback {
        return Ok(PaginatedCallbackAction::Handled);
    }

    if data == config.cancel_callback {
        let _ = bot.edit_message_reply_markup(msg.chat.id, msg.id).await;
        let process_name = config.running_process_key.translate(Arc::clone(ctx));
        let cancel_msg = i18n::commands::CANCEL_OK.translate_with_args(
            ctx.clone(),
            &maplit::hashmap! { i18n::args::PROCESS.into() => process_name.into() },
        );
        bot.send_message(msg.chat.id, cancel_msg).await?;
        dialogue.exit().await?;
        return Ok(PaginatedCallbackAction::Handled);
    }

    let value = data.strip_prefix(config.prefix).unwrap_or("").to_owned();
    if value.is_empty() {
        tracing::warn!("Empty value in callback data: {data:?}");
        return Ok(PaginatedCallbackAction::Handled);
    }

    // Check for page navigation.
    if let Some(page_str) = value.strip_prefix(&format!("{PAGE_MARKER}:")) {
        if let Ok(page) = page_str.parse::<usize>() {
            return Ok(PaginatedCallbackAction::PageChange {
                page,
                msg: Box::new(msg),
            });
        }
        return Ok(PaginatedCallbackAction::Handled);
    }

    // Edit the original message to show which option was selected.
    if let Some(text) = msg.text() {
        let _ = bot
            .edit_message_text(msg.chat.id, msg.id, format!("{text}\n✓ {value}"))
            .await;
    }

    Ok(PaginatedCallbackAction::Selection {
        value,
        msg: Box::new(msg),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, consts::BACK_LABEL};

    fn ctx() -> Arc<Mutex<Context>> {
        Arc::new(Mutex::new(Context::default()))
    }

    fn make_cfg<'a>(
        items: &'a [PickerItem],
        page: usize,
        columns: usize,
        action_buttons: &'a [InlineKeyboardButton],
        ctx: Arc<Mutex<Context>>,
    ) -> PaginatedKeyboardConfig<'a> {
        PaginatedKeyboardConfig {
            items,
            page,
            columns,
            rows_per_page: DEFAULT_ROWS_PER_PAGE,
            prefix: "pre:",
            cancel_callback: "pre:__cancel__",
            noop_callback: "pre:__noop__",
            action_buttons,
            show_cancel: true,
            ctx,
        }
    }

    #[test]
    fn paginated_keyboard_returns_none_for_empty_items() {
        let result = paginated_keyboard(make_cfg(&[], 0, 1, &[], ctx()));
        assert!(result.is_none());
    }

    #[test]
    fn paginated_keyboard_single_page_no_nav() {
        let items: Vec<PickerItem> = (1..=5)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        let kb = paginated_keyboard(make_cfg(&items, 0, 1, &[], ctx())).unwrap();
        // 5 item rows + 1 cancel row = 6 rows, no nav row
        assert_eq!(kb.inline_keyboard.len(), 6);
        assert_eq!(kb.inline_keyboard[0][0].text, "Item 1");
    }

    #[test]
    fn paginated_keyboard_multi_page_has_nav() {
        let items: Vec<PickerItem> = (1..=8)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        // rows_per_page=5, columns=1 → 5 items/page → 2 pages
        // Page 0: 5 items + nav row + cancel row = 7 rows
        let kb = paginated_keyboard(make_cfg(&items, 0, 1, &[], ctx())).unwrap();
        assert_eq!(kb.inline_keyboard.len(), 7);
        // Nav row (index 5) has 3 buttons: noop, "1/2", next
        let nav_row = &kb.inline_keyboard[5];
        assert_eq!(nav_row.len(), 3);
        assert_eq!(nav_row[1].text, "1/2");
        assert_eq!(nav_row[2].text, NEXT_LABEL);
    }

    #[test]
    fn paginated_keyboard_last_page() {
        let items: Vec<PickerItem> = (1..=8)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        // rows_per_page=5, columns=1 → 5 items/page → page 1 has 3 items
        // Page 1: 3 items + nav row + cancel row = 5 rows
        let kb = paginated_keyboard(make_cfg(&items, 1, 1, &[], ctx())).unwrap();
        assert_eq!(kb.inline_keyboard.len(), 5);
        // Nav row (index 3) has prev button
        let nav_row = &kb.inline_keyboard[3];
        assert_eq!(nav_row[0].text, BACK_LABEL);
        assert_eq!(nav_row[1].text, "2/2");
    }

    #[test]
    fn paginated_keyboard_page_clamped_to_max() {
        let items: Vec<PickerItem> = (1..=5)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        // Requesting page 99 on a single-page list should clamp to page 0
        let kb = paginated_keyboard(make_cfg(&items, 99, 1, &[], ctx())).unwrap();
        assert_eq!(kb.inline_keyboard.len(), 6);
        assert_eq!(kb.inline_keyboard[0][0].text, "Item 1");
    }

    #[test]
    fn paginated_keyboard_two_columns_layout() {
        let items: Vec<PickerItem> = (1..=5)
            .map(|i| PickerItem {
                label: format!("T{i}"),
                value: i.to_string(),
            })
            .collect();
        let kb = paginated_keyboard(make_cfg(&items, 0, 2, &[], ctx())).unwrap();
        // 5 items in 2-col grid: 3 item rows (2, 2, 1+spacer) + 1 cancel row = 4 rows
        assert_eq!(kb.inline_keyboard.len(), 4);
        assert_eq!(kb.inline_keyboard[0].len(), 2);
        assert_eq!(kb.inline_keyboard[0][0].text, "T1");
        assert_eq!(kb.inline_keyboard[0][1].text, "T2");
        // Last item row has one item + one spacer
        assert_eq!(kb.inline_keyboard[2][0].text, "T5");
        assert_eq!(kb.inline_keyboard[2][1].text, BLANK_LABEL);
    }

    #[test]
    fn paginated_keyboard_with_action_buttons() {
        let items: Vec<PickerItem> = (1..=3)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        let actions = vec![
            InlineKeyboardButton::callback("All".to_owned(), "pre:__all__".to_owned()),
            InlineKeyboardButton::callback("End".to_owned(), "pre:__end__".to_owned()),
        ];
        let kb = paginated_keyboard(make_cfg(&items, 0, 1, &actions, ctx())).unwrap();
        // 3 item rows + 1 action row + 1 cancel row = 5 rows
        assert_eq!(kb.inline_keyboard.len(), 5);
        // Action row is at index 3
        assert_eq!(kb.inline_keyboard[3][0].text, "All");
        assert_eq!(kb.inline_keyboard[3][1].text, "End");
    }

    #[test]
    fn paginated_keyboard_show_cancel_false_omits_cancel_row() {
        let items: Vec<PickerItem> = (1..=3)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        let kb = paginated_keyboard(PaginatedKeyboardConfig {
            items: &items,
            page: 0,
            columns: 1,
            rows_per_page: 5,
            prefix: "pre:",
            cancel_callback: "pre:__cancel__",
            noop_callback: "pre:__noop__",
            action_buttons: &[],
            show_cancel: false,
            ctx: ctx(),
        })
        .unwrap();
        // 3 item rows, no cancel row
        assert_eq!(kb.inline_keyboard.len(), 3);
    }

    #[test]
    fn paginated_keyboard_columns_zero_defaults_to_one() {
        let items: Vec<PickerItem> = (1..=3)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        let kb = paginated_keyboard(PaginatedKeyboardConfig {
            items: &items,
            page: 0,
            columns: 0,
            rows_per_page: 5,
            prefix: "pre:",
            cancel_callback: "pre:__cancel__",
            noop_callback: "pre:__noop__",
            action_buttons: &[],
            show_cancel: true,
            ctx: ctx(),
        })
        .unwrap();
        // columns=0 defaults to 1: 3 item rows + 1 cancel row = 4 rows
        assert_eq!(kb.inline_keyboard.len(), 4);
        // Each item row has exactly 1 button (no spacers)
        assert_eq!(kb.inline_keyboard[0].len(), DEFAULT_COLUMNS);
    }

    #[test]
    fn paginated_keyboard_rows_per_page_zero_defaults() {
        let items: Vec<PickerItem> = (1..=DEFAULT_ROWS_PER_PAGE + 1)
            .map(|i| PickerItem {
                label: format!("Item {i}"),
                value: i.to_string(),
            })
            .collect();
        let kb = paginated_keyboard(PaginatedKeyboardConfig {
            items: &items,
            page: 0,
            columns: 1,
            rows_per_page: 0,
            prefix: "pre:",
            cancel_callback: "pre:__cancel__",
            noop_callback: "pre:__noop__",
            action_buttons: &[],
            show_cancel: true,
            ctx: ctx(),
        })
        .unwrap();
        // rows_per_page=0 defaults to DEFAULT_ROWS_PER_PAGE:
        // DEFAULT_ROWS_PER_PAGE items shown + nav row + cancel row = DEFAULT_ROWS_PER_PAGE + 2 rows
        assert_eq!(kb.inline_keyboard.len(), DEFAULT_ROWS_PER_PAGE + 2);
        assert_eq!(kb.inline_keyboard[0][0].text, "Item 1");
        assert_eq!(
            kb.inline_keyboard[DEFAULT_ROWS_PER_PAGE - 1][0].text,
            format!("Item {}", DEFAULT_ROWS_PER_PAGE)
        );
    }
}
