use {config::Config, std::sync::LazyLock};

pub const TOKEN_MODE: &str = "token_mode";
pub const TOKEN: &str = "token";

pub const DB_HOST: &str = "db_host";
pub const DB_PORT: &str = "db_port";
pub const DB_USERNAME: &str = "db_username";
pub const DB_PASSWORD: &str = "db_password";
pub const DB_NAMESPACE: &str = "db_namespace";
pub const DB_DATABASE: &str = "db_database";

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap() // Panics if configurations cannot be loaded
});

pub enum TokenMode {
    File,
    Env,
    String,
}

impl TokenMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "file" => Self::File,
            "env" => Self::Env,
            "string" => Self::String,
            _ => panic!(
                "Invalid token mode: {}. Expected 'file', 'env', or 'string'",
                s
            ),
        }
    }
}

pub fn get_token() -> String {
    let token_mode = CONFIG.get::<String>(TOKEN_MODE).unwrap();
    let token = CONFIG.get::<String>(TOKEN).unwrap();

    match TokenMode::from_str(&token_mode) {
        TokenMode::File => std::fs::read_to_string(&token)
            .unwrap_or_else(|_| panic!("Token file '{token}' should be readable")),
        TokenMode::Env => std::env::var(&token)
            .unwrap_or_else(|_| panic!("Environment variable '{token}' should be set")),
        TokenMode::String => token,
    }
}
