pub const DECIMAL_SEP: char = '.';
pub const SPLIT_AMONG_ENTRIES_SEP: char = ';';
pub const SPLIT_AMONG_NAME_AMOUNT_SEP: char = ':';

pub const SPLIT_AMONG_REGEX_NAME_GRP: &str = "name";
pub const SPLIT_AMONG_REGEX_AMOUNT_GRP: &str = "amount";
pub const SPLIT_AMONG_REGEX_PERCENTAGE_GRP: &str = "percentage";

pub const ALL_KWORD: &str = "all";
pub const END_KWORD: &str = "end";
pub const RESERVED_KWORDS: [&str; 2] = [ALL_KWORD, END_KWORD];
pub const INVALID_CHARS: [char; 3] = [SPLIT_AMONG_ENTRIES_SEP, SPLIT_AMONG_NAME_AMOUNT_SEP, ','];

pub const MIN_SIMILARITY_SCORE: f32 = 0.25;
