/// Result of dispatching a command to its handler.
///
/// `Success` indicates the command completed successfully; the calling
/// dialogue should send the message and exit.
///
/// `Failure` indicates a user-visible failure such as a validation or
/// lookup mismatch (e.g. "traveler not found", "language not available"),
/// or a non-fatal error raised during dispatch; the calling dialogue
/// should send the message and stay alive so the user can supply another
/// value or `/cancel`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    Success(String),
    Failure(String),
}

impl CommandOutcome {
    pub fn message(&self) -> &str {
        match self {
            Self::Success(m) | Self::Failure(m) => m,
        }
    }

    pub fn into_message(self) -> String {
        match self {
            Self::Success(m) | Self::Failure(m) => m,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }
}
