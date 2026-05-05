macro_rules! variant_to_string {
    ($enum:ident::$variant:ident) => {
        stringify!($variant).to_lowercase()
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
            chat_id = ?q.regular_message().map(|m| m.chat.id),
            sender_id = %q.from.id,
            data = ?q.data,
        ),
    )
    ];
}
