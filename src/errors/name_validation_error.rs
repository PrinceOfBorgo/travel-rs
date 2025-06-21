use std::{error::Error, fmt::Display};

use maplit::hashmap;

use crate::i18n::{self, Translate, TranslateWithArgs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameValidationError {
    StartsWithSlash(String),
    InvalidCharacter(String, char),
    ReservedKeyword(String),
}

impl Translate for NameValidationError {
    fn translate_with_indent(
        &self,
        ctx: std::sync::Arc<std::sync::Mutex<crate::Context>>,
        indent_lvl: usize,
    ) -> String {
        match self {
            NameValidationError::StartsWithSlash(name) => {
                i18n::errors::NAME_VALIDATION_ERROR_STARTS_WITH_SLASH.translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.into()},
                    indent_lvl,
                )
            }
            NameValidationError::InvalidCharacter(name, char) => {
                i18n::errors::NAME_VALIDATION_ERROR_INVALID_CHAR.translate_with_args_indent(
                    ctx,
                    &hashmap! {
                        i18n::args::NAME.into() => name.into(),
                        i18n::args::CHAR.into() => char.to_string().into()
                    },
                    indent_lvl,
                )
            }
            NameValidationError::ReservedKeyword(name) => {
                i18n::errors::NAME_VALIDATION_ERROR_RESERVED_KEYWORD.translate_with_args_indent(
                    ctx,
                    &hashmap! {i18n::args::NAME.into() => name.into()},
                    indent_lvl,
                )
            }
        }
    }
}

impl Display for NameValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.translate_default())
    }
}

impl Error for NameValidationError {}
