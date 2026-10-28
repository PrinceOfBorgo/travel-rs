use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use std::str::FromStr;

use rust_decimal::Decimal;
use rusty_money::{FormattableCurrency, LocalFormat, Locale, Money, Round, crypto, iso};

use crate::Context;

/// Derives a flag emoji from a currency code by interpreting its first two
/// characters as an ISO 3166 country code and converting them into the
/// matching pair of Unicode regional indicator symbols.
///
/// - Works for the vast majority of ISO 4217 codes (country code + currency
///   letter), e.g. `USD` → 🇺🇸, `EUR` → 🇪🇺, `JPY` → 🇯🇵.
/// - Returns `None` for codes that don't map to a single country: anything
///   shorter than two letters, codes containing non-ASCII-letter characters,
///   and X-prefixed codes (precious metals like `XAU`, supranational
///   currencies like `XDR`/`XOF`, crypto codes, etc.).
fn currency_flag(code: &str) -> Option<String> {
    let bytes = code.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let (a, b) = (bytes[0], bytes[1]);
    if !a.is_ascii_alphabetic() || !b.is_ascii_alphabetic() {
        return None;
    }
    let a = a.to_ascii_uppercase();
    let b = b.to_ascii_uppercase();
    if a == b'X' {
        // X-prefixed codes are reserved for non-country uses (precious
        // metals, supranational currencies, crypto, ...): no flag.
        return None;
    }
    let to_regional = |c: u8| char::from_u32(0x1F1E6 + u32::from(c - b'A')).unwrap();
    Some([to_regional(a), to_regional(b)].iter().collect())
}

/// Currency symbol fetched from [`rusty_money`] (ISO first, then crypto).
fn currency_symbol(code: &str) -> Option<&'static str> {
    iso::find(code)
        .map(|c| c.symbol)
        .or_else(|| crypto::find(code).map(|c| c.symbol))
}

/// Renders a human-friendly currency label, e.g. `🇺🇸 USD ($)`.
///
/// - The flag is derived from the country-code prefix (see
///   [`currency_flag`]); codes without a meaningful flag (X-prefixed,
///   crypto, ...) are rendered without one.
/// - The symbol comes from [`rusty_money`] and is omitted when missing or
///   identical to the code.
pub fn currency_label(code: &str) -> String {
    let symbol = currency_symbol(code).unwrap_or("");
    let prefix = match currency_flag(code) {
        Some(flag) => format!("{flag} {code}"),
        None => code.to_owned(),
    };
    if symbol.is_empty() || symbol == code {
        prefix
    } else {
        format!("{prefix} ({symbol})")
    }
}

/// ISO 4217 minor-unit exponent and locale format of a currency (`2`/en-US if unknown).
fn currency_format(code: &str) -> (u32, LocalFormat) {
    let (exponent, locale) = iso::find(code)
        .map(|c| (c.exponent(), c.locale()))
        .or_else(|| crypto::find(code).map(|c| (c.exponent(), c.locale())))
        .unwrap_or((2, Locale::EnUs));
    (exponent, LocalFormat::from_locale(locale))
}

/// Maps Arabic-Indic, Persian and Devanagari digits to ASCII; other chars are unchanged.
fn to_ascii_digit(c: char) -> char {
    let zero = match c {
        '\u{0660}'..='\u{0669}' => 0x0660, // Arabic-Indic digits
        '\u{06F0}'..='\u{06F9}' => 0x06F0, // Persian digits
        '\u{0966}'..='\u{096F}' => 0x0966, // Devanagari digits
        _ => return c,
    };
    char::from(b'0' + (c as u32 - zero) as u8)
}

/// Parses a user-typed amount, accepting both `.` and `,` as decimal separator.
///
/// - Whitespace, apostrophes and the Arabic thousands separator `٬` are always
///   treated as thousands separators.
/// - The Arabic decimal separator `٫` is always a decimal separator.
/// - Arabic-Indic, Persian and Devanagari digits are accepted.
/// - If both `.` and `,` appear, the last one is the decimal separator.
/// - If one of them appears more than once, it is a thousands separator.
/// - A single separator is a decimal separator, except when it is the
///   currency's thousands separator, followed by exactly 3 digits and the
///   currency has fewer than 3 minor digits (e.g. `1.234` in EUR is `1234`,
///   `1,234` in USD is `1234`, while `1,23`/`1.23` is `1.23` in both).
/// - Thousands groups must be well formed (3 digits, or 2 for Indian-style
///   inner groups).
pub fn parse_amount(text: &str, currency_code: &str) -> Option<Decimal> {
    let (exponent, format) = currency_format(currency_code);
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '\'' | '’' | '٬'))
        // The currency's decimal separator is never ambiguous for the logic below.
        .map(|c| {
            if c == '٫' {
                format.exponent_separator
            } else {
                to_ascii_digit(c)
            }
        })
        .collect();
    let (sign, body) = match cleaned.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", cleaned.strip_prefix('+').unwrap_or(&cleaned)),
    };
    if body.is_empty()
        || !body
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
    {
        return None;
    }

    let dots = body.matches('.').count();
    let commas = body.matches(',').count();
    // `None` means the number has no decimal part.
    let decimal_idx = match (dots, commas) {
        (0, 0) => None,
        // Only one kind of separator.
        (_, 0) | (0, _) => {
            let sep = if dots > 0 { '.' } else { ',' };
            let idx = body.rfind(sep)?;
            let (int_part, frac_part) = (&body[..idx], &body[idx + 1..]);
            // Ambiguous `x.yyy`/`x,yyy`: read as grouping only if the separator is the
            // currency's thousands one, 3 digits can't be minor units, and the integer
            // part is a valid leading group (a leading `0` means a decimal like `0,500`).
            let looks_grouped = sep == format.digit_separator
                && frac_part.len() == 3
                && exponent < 3
                && (1..=3).contains(&int_part.len())
                && !int_part.starts_with('0');
            // A repeated separator can only be a thousands separator.
            if dots + commas > 1 || looks_grouped {
                None
            } else {
                Some(idx)
            }
        }
        // Both kinds: the last one is the decimal separator and must be unique.
        _ => {
            let idx = body.rfind(['.', ','])?;
            let sep = &body[idx..=idx];
            // e.g. `1.234,5,6`: the decimal separator also appears as a grouping one.
            if body.matches(sep).count() > 1 {
                return None;
            }
            Some(idx)
        }
    };

    let (int_part, frac_part) = match decimal_idx {
        Some(idx) => (&body[..idx], Some(&body[idx + 1..])),
        None => (body, None),
    };
    // Validate thousands grouping (3-digit groups, 2-digit inner groups for en-IN).
    let groups: Vec<&str> = int_part.split(['.', ',']).collect();
    if let Some((first, rest)) = groups.split_first()
        && !rest.is_empty()
    {
        let last = rest.len() - 1;
        // Leading group has 1-3 digits; the last group always has 3 (e.g. `1,00,000`).
        let well_formed = (1..=3).contains(&first.len())
            && rest
                .iter()
                .enumerate()
                .all(|(i, g)| g.len() == 3 || (i < last && g.len() == 2));
        if !well_formed {
            return None;
        }
    }

    let int_digits: String = groups.concat();
    // Rebuild a canonical `[-]digits[.digits]` string for `Decimal::from_str`.
    let normalized = match frac_part {
        // Trailing separator with no digits, e.g. `1.`.
        Some("") => return None,
        // Missing integer part, e.g. `.5`.
        Some(frac) if int_digits.is_empty() => format!("{sign}0.{frac}"),
        Some(frac) => format!("{sign}{int_digits}.{frac}"),
        None => format!("{sign}{int_digits}"),
    };
    Decimal::from_str(&normalized).ok()
}

pub enum MoneyWrapper<'a> {
    Iso(Money<'a, iso::Currency>),
    Crypto(Money<'a, crypto::Currency>),
    Other { amount: Decimal, currency: String },
}

impl MoneyWrapper<'_> {
    pub fn new(amount: Decimal, currency_code: &str) -> Self {
        if let Some(currency) = iso::find(currency_code) {
            Self::Iso(Money::from_decimal(amount, currency))
        } else if let Some(currency) = crypto::find(currency_code) {
            Self::Crypto(Money::from_decimal(amount, currency))
        } else {
            Self::Other {
                amount,
                currency: currency_code.to_owned(),
            }
        }
    }

    pub fn new_with_context(amount: Decimal, ctx: Arc<Mutex<Context>>) -> Self {
        let currency_code = &ctx.lock().expect("Failed to lock context").currency.clone();
        Self::new(amount, currency_code)
    }

    pub fn round_value(&self) -> Decimal {
        match self {
            MoneyWrapper::Iso(money) => *money
                .round(money.currency().exponent, Round::HalfEven)
                .amount(),
            MoneyWrapper::Crypto(money) => *money
                .round(money.currency().exponent, Round::HalfEven)
                .amount(),
            MoneyWrapper::Other {
                amount,
                currency: _,
            } => *amount,
        }
    }

    fn fmt_no_digit_separators<T: rusty_money::FormattableCurrency>(
        money: &Money<'_, T>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let currency = money.currency();
        let format = rusty_money::LocalFormat::from_locale(currency.locale());

        let mut format_params = rusty_money::Params {
            digit_separator: format.digit_separator,
            exponent_separator: format.exponent_separator,
            separator_pattern: &[],
            rounding: Some(currency.exponent()),
            symbol: Some(currency.symbol()),
            code: Some(currency.code()),
            ..Default::default()
        };

        if currency.symbol_first() {
            format_params.positions = &[
                rusty_money::Position::Sign,
                rusty_money::Position::Symbol,
                rusty_money::Position::Amount,
            ];
            write!(f, "{}", rusty_money::Formatter::money(money, format_params))
        } else {
            format_params.positions = &[
                rusty_money::Position::Sign,
                rusty_money::Position::Amount,
                rusty_money::Position::Symbol,
            ];
            write!(f, "{}", rusty_money::Formatter::money(money, format_params))
        }
    }
}

impl Display for MoneyWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoneyWrapper::Iso(money) => Self::fmt_no_digit_separators(money, f),
            MoneyWrapper::Crypto(money) => Self::fmt_no_digit_separators(money, f),
            MoneyWrapper::Other { amount, currency } => {
                write!(f, "{}", amount.to_string() + " " + currency)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Option<Decimal> {
        Some(Decimal::from_str(s).unwrap())
    }

    #[test]
    fn parse_amount_plain() {
        assert_eq!(parse_amount("123", "EUR"), dec("123"));
        assert_eq!(parse_amount(" -25.2 ", "USD"), dec("-25.2"));
        assert_eq!(parse_amount("+7", "USD"), dec("7"));
        assert_eq!(parse_amount(".5", "USD"), dec("0.5"));
    }

    #[test]
    fn parse_amount_both_decimal_separators() {
        for code in ["EUR", "USD", "JPY", "BTC", "ZZZ"] {
            assert_eq!(parse_amount("123,45", code), dec("123.45"), "{code}");
            assert_eq!(parse_amount("123.45", code), dec("123.45"), "{code}");
        }
    }

    #[test]
    fn parse_amount_ambiguous_uses_currency_format() {
        assert_eq!(parse_amount("1.234", "EUR"), dec("1234"));
        assert_eq!(parse_amount("1,234", "EUR"), dec("1.234"));
        assert_eq!(parse_amount("1,234", "USD"), dec("1234"));
        assert_eq!(parse_amount("1.234", "USD"), dec("1.234"));
        assert_eq!(parse_amount("0,500", "USD"), dec("0.5"));
        assert_eq!(parse_amount("1,234", "BTC"), dec("1.234"));
    }

    #[test]
    fn parse_amount_thousands_separators() {
        assert_eq!(parse_amount("1.234,56", "USD"), dec("1234.56"));
        assert_eq!(parse_amount("1,234.56", "EUR"), dec("1234.56"));
        assert_eq!(parse_amount("1.234.567", "EUR"), dec("1234567"));
        assert_eq!(parse_amount("1,234,567", "USD"), dec("1234567"));
        assert_eq!(parse_amount("1 234,5", "EUR"), dec("1234.5"));
        assert_eq!(parse_amount("1'234.5", "CHF"), dec("1234.5"));
        assert_eq!(parse_amount("1,00,000", "INR"), dec("100000"));
        assert_eq!(parse_amount("12,34,56,789", "INR"), dec("123456789"));
        assert_eq!(
            parse_amount("12,34,56,789.123", "INR"),
            dec("123456789.123")
        );
    }

    #[test]
    fn parse_amount_non_latin() {
        assert_eq!(parse_amount("١٬٢٣٤٫٥", "EGP"), dec("1234.5"));
        assert_eq!(parse_amount("۱۲٫۵", "IRR"), dec("12.5"));
        assert_eq!(parse_amount("1٫234", "EUR"), dec("1.234"));
        assert_eq!(parse_amount("१,००,०००", "INR"), dec("100000"));
    }

    #[test]
    fn parse_amount_invalid() {
        for input in [
            "",
            "-",
            "abc",
            "12a",
            "1.",
            "1.2.3",
            "1,234,56.7.8",
            "1.234,5,6",
            "1234.567,8",
            "1,2,345",
        ] {
            assert_eq!(parse_amount(input, "EUR"), None, "{input}");
        }
    }
}
