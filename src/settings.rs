use config::Config;
use serde::Deserialize;
use std::{path::PathBuf, sync::LazyLock};
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
        .build()
        .unwrap();
    conf.try_deserialize().unwrap() // Panics if configurations cannot be loaded
});

#[cfg(test)]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    let conf = Config::builder()
        .add_source(config::File::with_name("config/profiles/unit-tests"))
        .build()
        .unwrap();
    conf.try_deserialize().unwrap() // Panics if configurations cannot be loaded
});

enum TokenSource {
    File,
    Env,
    String,
}

impl TokenSource {
    fn from_str(s: &str) -> Self {
        match s {
            "file" => Self::File,
            "env" => Self::Env,
            "string" => Self::String,
            _ => panic!("Invalid token source: {s}. Expected 'file', 'env', or 'string'"),
        }
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
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub address: String,
    pub username: String,
    pub password: String,
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
    pub logging: Logging,
    pub bot: Bot,
    pub database: Database,
    pub i18n: I18n,
}

impl Settings {
    pub fn token_value(&self) -> String {
        let token_source = &self.bot.token_source;
        let token = &self.bot.token;

        match TokenSource::from_str(token_source) {
            TokenSource::File => std::fs::read_to_string(token)
                .unwrap_or_else(|_| panic!("Token file '{token}' should be readable")),
            TokenSource::Env => std::env::var(token)
                .unwrap_or_else(|_| panic!("Environment variable '{token}' should be set")),
            TokenSource::String => token.clone(),
        }
    }
}
