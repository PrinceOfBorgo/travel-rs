//! Shared inline-keyboard layout and callback-handling utilities for
//! interactive dialogues.

mod callback;
mod paginated;
mod travelers;

pub use callback::{CallbackAction, CallbackConfig, handle_callback_prelude};
pub use paginated::{
    DEFAULT_COLUMNS, DEFAULT_ROWS_PER_PAGE, PaginatedCallbackAction, PaginatedKeyboardConfig,
    PickerItem, handle_paginated_callback, paginated_keyboard,
};
pub use travelers::{TravelersKeyboardConfig, travelers_keyboard};
