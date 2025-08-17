use config::Config;
use serde::Deserialize;
use std::{path::PathBuf, sync::LazyLock};
use teloxide::types::ChatId;
use unic_langid::LanguageIdentifier;

use crate::ARGS;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("config/config"))
        .build()
        .unwrap() // Panics if configurations cannot be loaded
});

#[cfg(not(test))]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    // Retrieve the profile from command line arguments or fallback to the configuration file
    let profile = ARGS
        .profile
        .clone()
        .unwrap_or_else(|| CONFIG.get_string("profile").unwrap());
    let conf = Config::builder()
        .add_source(config::File::with_name(&format!(
            "config/profiles/{profile}"
        )))
        .set_override("profile", profile)
        .unwrap() // Add profile to the configuration
        .build()
        .unwrap();
    conf.try_deserialize().unwrap() // Panics if configurations cannot be loaded
});

#[cfg(test)]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    let conf = Config::builder()
        .add_source(config::File::with_name("config/profiles/unit-tests"))
        .set_override("profile", "unit-tests")
        .unwrap() // Add profile to the configuration
        .build()
        .unwrap();
    conf.try_deserialize().unwrap() // Panics if configurations cannot be loaded
});

enum PropertySource {
    File,
    Env,
    String,
}

impl PropertySource {
    fn from_str(s: &str) -> Self {
        match s {
            "file" => Self::File,
            "env" => Self::Env,
            "string" => Self::String,
            _ => panic!("Invalid property source: {s}. Expected 'file', 'env', or 'string'"),
        }
    }
}

#[derive(Deserialize)]
pub struct HiddenString(pub String);

impl std::fmt::Display for HiddenString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*** HIDDEN ***")
    }
}

impl std::fmt::Debug for HiddenString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, Deserialize)]
pub struct Logging {
    pub path: String,
    pub file_name_prefix: String,
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct Bot {
    pub token_source: String,
    pub token: HiddenString,
    pub chat_whitelist_source: Option<String>,
    pub chat_whitelist: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub address: String,
    pub username: String,
    pub password: HiddenString,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct I18n {
    pub default_locale: LanguageIdentifier,
    pub locales_path: PathBuf,
    pub default_currency: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub profile: String,
    pub logging: Logging,
    pub bot: Bot,
    pub database: Database,
    pub i18n: I18n,
}

impl Settings {
    pub fn token_value(&self) -> String {
        let token_source = &self.bot.token_source;
        let token = &self.bot.token.0;

        match PropertySource::from_str(token_source) {
            PropertySource::File => std::fs::read_to_string(token)
                .unwrap_or_else(|_| panic!("Token file '{token}' should be readable")),
            PropertySource::Env => std::env::var(token)
                .unwrap_or_else(|_| panic!("Environment variable '{token}' should be set")),
            PropertySource::String => token.clone(),
        }
    }

    pub fn chat_whitelist_value(&self) -> Vec<ChatId> {
        let Some(chat_whitelist_source) = &self.bot.chat_whitelist_source else {
            return Vec::new(); // No whitelist source specified, return empty vector
        };
        let Some(chat_whitelist) = &self.bot.chat_whitelist else {
            return Vec::new(); // No whitelist specified, return empty vector
        };

        let content: String = match PropertySource::from_str(chat_whitelist_source) {
            PropertySource::File => std::fs::read_to_string(chat_whitelist)
                .unwrap_or_else(|_| panic!("Whitelist file '{chat_whitelist}' should be readable")),
            PropertySource::Env => std::env::var(chat_whitelist).unwrap_or_else(|_| {
                panic!("Environment variable '{chat_whitelist}' should be set")
            }),
            PropertySource::String => chat_whitelist.clone(),
        };

        // Parse comma or whitespace separated chat ids into Vec<ChatId>
        content
            .split(&[',', ';', ' ', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<i64>()
                    .map(ChatId)
                    .unwrap_or_else(|_| panic!("Invalid chat id in whitelist: {s}"))
            })
            .collect()
    }
}
