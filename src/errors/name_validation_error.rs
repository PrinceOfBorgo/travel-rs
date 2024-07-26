use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum NameValidationError {
    StartsWithSlash(String),
    InvalidCharacter(String, char),
    ReservedKeyword(String),
}

impl Display for NameValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameValidationError::StartsWithSlash(s) => {
                write!(f, "The name `{s}` starts with a slash `/`")
            }
            NameValidationError::InvalidCharacter(s, c) => {
                write!(f, "The name `{s}` contains an invalid character: `{c}`")
            }
            NameValidationError::ReservedKeyword(s) => {
                write!(f, "`{s}` is a reserved keyword")
            }
        }
    }
}

impl Error for NameValidationError {}
