pub const DECIMAL_SEP: char = '.';
pub const SPLIT_AMONG_ENTRIES_SEP: char = ';';
pub const SPLIT_AMONG_NAME_AMOUNT_SEP: char = ':';

pub const SPLIT_AMONG_REGEX_NAME_GRP: &str = "name";
pub const SPLIT_AMONG_REGEX_AMOUNT_GRP: &str = "amount";
pub const SPLIT_AMONG_REGEX_PERCENTAGE_GRP: &str = "percentage";

pub const ALL_KWORD: &str = "all";
pub const END_KWORD: &str = "end";
pub const RESERVED_KWORDS: [&str; 2] = [ALL_KWORD, END_KWORD];

pub const BLANK_LABEL: &str = "\u{2800}"; // Braille Pattern Blank — invisible but non-empty
pub const BACK_LABEL: &str = "◀";
pub const NEXT_LABEL: &str = "▶";

pub const NAME_INVALID_CHARS: [char; 4] = [
    SPLIT_AMONG_ENTRIES_SEP,
    SPLIT_AMONG_NAME_AMOUNT_SEP,
    '"',
    ',',
];

/// Minimum similarity score (between 0 and 1) for a string to be considered similar enough to
/// another for the purposes of matching user input to existing travelers or expenses.
pub const MIN_SIMILARITY_SCORE: f32 = 0.25;

pub const LOG_DEBUG_START: &str = "START";
pub const LOG_DEBUG_SUCCESS: &str = "SUCCESS";

/// Text keywords accepted as confirmation / denial in the Confirm state.
pub const CONFIRM_YES_KWORDS: &[&str] = &["yes", "y"];
pub const CONFIRM_NO_KWORDS: &[&str] = &["no", "n"];

/// Monotonically increasing counter used as a per-span trace identifier.
/// Each call returns a unique value within the process lifetime.
static TRACE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub fn next_trace_id() -> u64 {
    TRACE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
