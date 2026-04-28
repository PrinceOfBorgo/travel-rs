use std::str::FromStr;

/// Wrapper for an optional command argument.
///
/// `Missing` means the user invoked the command without supplying a value
/// (e.g. `/addtraveler` with no name); the dispatcher then routes the
/// message to a dialogue that prompts for it interactively. `Provided`
/// carries the parsed value.
///
/// Because [`CommandArg<T>`] implements [`FromStr`] whenever `T` does,
/// teloxide's default command parser handles it natively.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub enum CommandArg<T> {
    #[default]
    Missing,
    Provided(T),
}

impl<T> CommandArg<T> {
    /// Returns `true` if the argument was omitted by the user.
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Unwraps a [`CommandArg::Provided`]. Panics if the value is `Missing`,
    /// signalling that the dispatcher failed to route the missing-argument
    /// case to its dialogue.
    pub fn expect_provided(self, command: &str) -> T {
        match self {
            Self::Provided(value) => value,
            Self::Missing => unreachable!(
                "/{command} reached command_reply with no argument; the dispatcher \
                 should have routed the empty case to its dialogue."
            ),
        }
    }
}

impl<T> From<T> for CommandArg<T> {
    fn from(value: T) -> Self {
        Self::Provided(value)
    }
}

impl<T: FromStr> FromStr for CommandArg<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Ok(Self::Missing)
        } else {
            T::from_str(trimmed).map(Self::Provided)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_empty_is_missing() {
        let arg: CommandArg<i64> = "".parse().unwrap();
        assert_eq!(arg, CommandArg::Missing);
    }

    #[test]
    fn from_str_whitespace_is_missing() {
        let arg: CommandArg<i64> = "   ".parse().unwrap();
        assert_eq!(arg, CommandArg::Missing);
    }

    #[test]
    fn from_str_value_is_provided() {
        let arg: CommandArg<i64> = "42".parse().unwrap();
        assert_eq!(arg, CommandArg::Provided(42));
    }

    #[test]
    fn from_str_invalid_propagates_error() {
        let result: Result<CommandArg<i64>, _> = "abc".parse();
        assert!(result.is_err());
    }

    #[test]
    fn is_missing() {
        assert!(CommandArg::<i64>::Missing.is_missing());
        assert!(!CommandArg::Provided(1_i64).is_missing());
    }

    #[test]
    fn expect_provided_returns_inner() {
        assert_eq!(CommandArg::Provided(7_i64).expect_provided("test"), 7);
    }

    #[test]
    #[should_panic(expected = "/test reached command_reply with no argument")]
    fn expect_provided_panics_on_missing() {
        let _ = CommandArg::<i64>::Missing.expect_provided("test");
    }
}
