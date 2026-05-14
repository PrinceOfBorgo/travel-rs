//! Traveler-picker inline keyboard.

use crate::{Context, traveler::Traveler};
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::types::{ChatId, InlineKeyboardMarkup};

use super::paginated::{
    DEFAULT_ROWS_PER_PAGE, PaginatedKeyboardConfig, PickerItem, paginated_keyboard,
};

/// Number of traveler buttons per row in the inline keyboard.
const TRAVELERS_PER_ROW: usize = 2;

/// Configuration for building a traveler-picker inline keyboard.
pub struct TravelersKeyboardConfig<'a> {
    /// Database connection.
    pub db: Arc<Surreal<Any>>,
    /// Chat to load travelers from.
    pub chat_id: ChatId,
    /// Callback-data prefix prepended to each traveler index.
    pub prefix: &'a str,
    /// Full callback data for the cancel button.
    pub cancel_callback: &'a str,
    /// Full callback data for blank spacer buttons.
    pub noop_callback: &'a str,
    /// Whether to show the cancel button row.
    pub show_cancel: bool,
    /// Shared context for i18n.
    pub ctx: Arc<Mutex<Context>>,
}

/// Loads the chat's travelers and builds a paginated two-column grid keyboard
/// with one button per name (using `prefix` as the callback-data prefix).
/// When `show_cancel` is `true`, a cancel row is appended.
///
/// Returns `None` if no travelers exist or if the DB query fails.
pub async fn travelers_keyboard(
    config: TravelersKeyboardConfig<'_>,
) -> Option<InlineKeyboardMarkup> {
    let travelers = Traveler::db_select(config.db, config.chat_id).await.ok()?;
    if travelers.is_empty() {
        return None;
    }
    let items: Vec<PickerItem> = travelers
        .into_iter()
        .map(|t| {
            PickerItem {
                label: t.name.to_string(),
                value: t.number.to_string(),
            }
        })
        .collect();

    paginated_keyboard(PaginatedKeyboardConfig {
        items: &items,
        page: 0,
        columns: TRAVELERS_PER_ROW,
        rows_per_page: DEFAULT_ROWS_PER_PAGE,
        prefix: config.prefix,
        cancel_callback: config.cancel_callback,
        noop_callback: config.noop_callback,
        action_buttons: &[],
        show_cancel: config.show_cancel,
        ctx: config.ctx,
    })
}
