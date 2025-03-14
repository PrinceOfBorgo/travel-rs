use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use rust_decimal::Decimal;
use rusty_money::{Money, Round, crypto, iso};

use crate::Context;

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
            separator_pattern: vec![],
            rounding: Some(currency.exponent()),
            symbol: Some(currency.symbol()),
            code: Some(currency.code()),
            ..Default::default()
        };

        if currency.symbol_first() {
            format_params.positions = vec![
                rusty_money::Position::Sign,
                rusty_money::Position::Symbol,
                rusty_money::Position::Amount,
            ];
            write!(f, "{}", rusty_money::Formatter::money(money, format_params))
        } else {
            format_params.positions = vec![
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
