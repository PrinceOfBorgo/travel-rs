use macro_rules_attribute::attribute_alias;

attribute_alias! {
    #[apply(trace_skip_all)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip_all,
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from().unwrap().id
        )
    )
    ];
}

attribute_alias! {
    #[apply(trace_command)] =
    #[tracing::instrument(
        err(level = Level::ERROR),
        ret(level = Level::DEBUG),
        skip(msg),
        fields(
            chat_id = %msg.chat.id,
            sender_id = %msg.from().unwrap().id
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
            sender_id = %msg.from().unwrap().id
        )
    )
    ];
}
