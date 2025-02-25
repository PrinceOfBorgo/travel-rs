use config::Config;
use serde::Deserialize;
use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("config/config"))
        .build()
        .unwrap() // Panics if configurations cannot be loaded
});

pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    let profile = CONFIG.get_string("profile").unwrap();
    Config::builder()
        .add_source(config::File::with_name(&format!(
            "config/profiles/{profile}"
        )))
        .build()
        .and_then(Config::try_deserialize)
        .unwrap() // Panics if configurations cannot be loaded
});

pub enum TokenSource {
    File,
    Env,
    String,
}

impl TokenSource {
    pub fn from_str(s: &str) -> Self {
        match s {
            "file" => Self::File,
            "env" => Self::Env,
            "string" => Self::String,
            _ => panic!(
                "Invalid token source: {}. Expected 'file', 'env', or 'string'",
                s
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Bot {
    pub token_source: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub host: String,
    pub port: usize,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub bot: Bot,
    pub database: Database,
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
