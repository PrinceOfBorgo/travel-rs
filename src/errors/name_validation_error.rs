use std::{error::Error, fmt::Display};

use maplit::hashmap;

use crate::i18n::{self, Translate, translate_with_args};

#[derive(Debug, Clone)]
pub enum NameValidationError {
    StartsWithSlash(String),
    InvalidCharacter(String, char),
    ReservedKeyword(String),
}

impl Translate for NameValidationError {
    fn translate(&self, ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>) -> String {
        match self {
            NameValidationError::StartsWithSlash(name) => translate_with_args(
                ctx,
                i18n::errors::NAME_VALIDATION_ERROR_STARTS_WITH_SLASH,
                &hashmap! {i18n::args::NAME.into() => name.into()},
            ),
            NameValidationError::InvalidCharacter(name, char) => translate_with_args(
                ctx,
                i18n::errors::NAME_VALIDATION_ERROR_INVALID_CHAR,
                &hashmap! {
                    i18n::args::NAME.into() => name.into(),
                    i18n::args::CHAR.into() => char.to_string().into()
                },
            ),
            NameValidationError::ReservedKeyword(name) => translate_with_args(
                ctx,
                i18n::errors::NAME_VALIDATION_ERROR_RESERVED_KEYWORD,
                &hashmap! {i18n::args::NAME.into() => name.into()},
            ),
        }
    }
}

impl Display for NameValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl Error for NameValidationError {}
