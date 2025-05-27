mod integration_test;

use crate::{Context, deps, handler_tree};
use std::sync::{Arc, Mutex};
use surrealdb::{Surreal, engine::any::Any};
use teloxide::{
    dptree::di::DependencySupplier,
    types::{Chat, ChatId},
};
use teloxide_tests::{MockBot, MockMessageText, mock_bot::DistributionKey};

pub(crate) struct TestBot {
    bot: MockBot<Box<dyn std::error::Error + Send + Sync>, DistributionKey>,
    chat_id: ChatId,
}

impl TestBot {
    /// Creates a new `TestBot` instance with a random chat ID.
    /// The `text` parameter is the initial message text.
    pub fn new(db_instance: Arc<Surreal<Any>>, text: &str) -> Self {
        let chat_id = ChatId(rand::random());
        Self::with_chat_id(db_instance, text, chat_id)
    }

    /// Creates a new `TestBot` instance with the specified chat ID.
    /// The `text` parameter is the initial message text.
    pub fn with_chat_id(db_instance: Arc<Surreal<Any>>, text: &str, chat_id: ChatId) -> Self {
        let mock_msg = mock_text_from_chat_id(text, chat_id);
        let mut bot = MockBot::new(mock_msg, handler_tree());
        bot.dependencies(deps(db_instance.clone()));
        Self { bot, chat_id }
    }

    pub fn chat_id(&self) -> ChatId {
        self.chat_id
    }

    /// Updates the bot with a new message.
    /// The `text` parameter is the new message text.
    pub fn update(&mut self, text: &str) {
        let mock_msg = mock_text_from_chat_id(text, self.chat_id);
        self.bot.update(mock_msg);
    }

    /// Dispatches the bot and checks the last message sent.
    /// The `response` parameter is the expected response text.
    pub async fn test_last_message(&mut self, response: &str) {
        self.bot.dispatch_and_check_last_text(response).await;
    }

    /// Dispatches the bot.
    pub async fn dispatch(&mut self) {
        self.bot.dispatch().await;
    }

    /// Dispatches the bot and returns the last message sent.
    /// Returns `None` if no messages were sent.
    pub async fn dispatch_and_last_message(&mut self) -> Option<String> {
        let bot = &mut self.bot;
        bot.dispatch().await;
        self.last_message()
    }

    /// Returns the last message sent.
    /// Returns `None` if no messages were sent.
    pub fn last_message(&self) -> Option<String> {
        let bot = &self.bot;
        let responses = bot.get_responses();
        responses
            .sent_messages
            .last()
            .and_then(|msg| msg.text())
            .map(|s| s.to_owned())
    }

    pub fn context(&self) -> Arc<Mutex<Context>> {
        let arc: Arc<Arc<Mutex<Context>>> = self.bot.dependencies.get();
        Arc::clone(&arc)
    }
}

fn mock_text_from_chat_id(text: &str, chat_id: ChatId) -> MockMessageText {
    let mock_msg = MockMessageText::new().text(text);
    let mock_chat = mock_msg.chat.clone();
    mock_msg.chat(Chat {
        id: chat_id,
        ..mock_chat
    })
}

pub mod helpers {
    use rust_decimal::Decimal;

    use super::*;

    #[inline]
    pub async fn add_traveler(bot: &mut TestBot, name: &str) {
        bot.update(&format!("/addtraveler {name}"));
        bot.dispatch().await;
    }

    #[inline]
    pub async fn add_expense(
        bot: &mut TestBot,
        description: &str,
        amount: Decimal,
        payer: &str,
        split: &[&str],
    ) {
        bot.update("/addexpense");
        bot.dispatch().await;
        // 1. Set description
        bot.update(description);
        bot.dispatch().await;
        // 2. Set amount
        bot.update(&amount.to_string());
        bot.dispatch().await;
        // 3. Set payer
        bot.update(payer);
        bot.dispatch().await;
        // 4. Split expense
        for s in split {
            bot.update(s);
            bot.dispatch().await;
        }
    }

    #[inline]
    pub async fn transfer(bot: &mut TestBot, sender: &str, receiver: &str, amount: Decimal) {
        bot.update(&format!("/transfer {sender} {receiver} {amount}"));
        bot.dispatch().await;
    }

    #[inline]
    pub async fn add_travelers_and_transfer(
        bot: &mut TestBot,
        sender: &str,
        receiver: &str,
        amount: Decimal,
    ) {
        // Add sender
        bot.update(&format!("/addtraveler {sender}"));
        bot.dispatch().await;

        // Add receiver
        bot.update(&format!("/addtraveler {receiver}"));
        bot.dispatch().await;

        // Transfer amount from sender to receiver
        bot.update(&format!("/transfer {sender} {receiver} {amount}"));
        bot.dispatch().await;
    }
}
