use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use rust_decimal::Decimal;
use rusty_money::{Money, Round, crypto, iso};

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
