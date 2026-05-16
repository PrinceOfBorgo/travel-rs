macro_rules! variant_to_string {
    ($enum:ident::$variant:ident) => {
        stringify!($variant).to_lowercase()
    };
}

/// Generates the standard callback-data constants for an inline-keyboard
/// dialogue from a single prefix string.
///
/// # Usage
///
/// ```ignore
/// // Picker dialogue (cancel + noop):
/// callback_consts!("setlang" => cancel, noop);
///
/// // Confirmation dialogue (confirm + deny):
/// callback_consts!("clrexp" => confirm, deny);
///
/// // Picker that transitions to a confirmation step:
/// callback_consts!("delexp" => cancel, noop, confirm, deny);
/// ```
///
/// # Generated constants
///
/// `CALLBACK_PREFIX` is always generated as `"{prefix}:"`.
/// Each sentinel keyword produces a `pub const`:
///
/// | Keyword   | Constant           | Value                     |
/// |-----------|--------------------|---------------------------|
/// | `cancel`  | `CANCEL_CALLBACK`  | `"{prefix}:__cancel__"`   |
/// | `noop`    | `NOOP_CALLBACK`    | `"{prefix}:__noop__"`     |
/// | `confirm` | `CONFIRM_CALLBACK` | `"{prefix}:__confirm__"`  |
/// | `deny`    | `DENY_CALLBACK`    | `"{prefix}:__deny__"`     |
macro_rules! callback_consts {
    ($prefix:literal => $($sentinel:ident),+ $(,)?) => {
        pub const CALLBACK_PREFIX: &str = concat!($prefix, ":");
        $(callback_consts!(@sentinel $prefix, $sentinel);)+
    };
    ($prefix:literal) => {
        pub const CALLBACK_PREFIX: &str = concat!($prefix, ":");
    };
    (@sentinel $prefix:literal, cancel) => {
        pub const CANCEL_CALLBACK: &str = concat!($prefix, ":__cancel__");
    };
    (@sentinel $prefix:literal, noop) => {
        pub const NOOP_CALLBACK: &str = concat!($prefix, ":__noop__");
    };
    (@sentinel $prefix:literal, confirm) => {
        pub const CONFIRM_CALLBACK: &str = concat!($prefix, ":__confirm__");
    };
    (@sentinel $prefix:literal, deny) => {
        pub const DENY_CALLBACK: &str = concat!($prefix, ":__deny__");
    };
}

#[cfg(test)]
/// This macro is used to define asynchronous test functions with a specific configuration.
/// It simplifies the creation of test functions by automatically applying the `tokio::test`
/// attribute with the specified flavor and worker threads.
macro_rules! test {
    ($title:ident, $($stmt:stmt)*) => {
        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn $title() {
            $($stmt)*
        }
    };
}

// ─── Tracing attribute aliases ───────────────────────────────────────────────

use macro_rules_attribute::attribute_alias;

attribute_alias! {
    #[apply(trace_skip_all)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip_all,
        fields(
            trace_id = crate::consts::next_trace_id(),
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id,
            msg_text = ?msg.text(),
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_command)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(msg, ctx),
        fields(
            trace_id = crate::consts::next_trace_id(),
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id,
            msg_text = ?msg.text(),
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_command_db)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(db, msg, ctx),
        fields(
            trace_id = crate::consts::next_trace_id(),
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id,
            msg_text = ?msg.text(),
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_state)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(bot, dialogue, msg),
        fields(
            trace_id = crate::consts::next_trace_id(),
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id,
            msg_text = ?msg.text(),
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_state_db)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(db, bot, dialogue, msg),
        fields(
            trace_id = crate::consts::next_trace_id(),
            chat_id = %msg.chat.id,
            sender_id = %msg.from.as_ref().unwrap().id,
            msg_text = ?msg.text(),
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_callback)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip_all,
        fields(
            trace_id = crate::consts::next_trace_id(),
            chat_id = ?q.regular_message().map(|m| m.chat.id),
            sender_id = %q.from.id,
            data = ?q.data,
        ),
    )
    ];
}
