use config::{Config, Source, Value, ValueKind};
use serde::Deserialize;
use std::{path::PathBuf, sync::LazyLock};
use teloxide::types::ChatId;
use unic_langid::LanguageIdentifier;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("config/config"))
        .build()
        .unwrap() // Panics if configurations cannot be loaded
});

#[cfg(not(test))]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    // Retrieve the profile from command line arguments or fallback to the configuration file
    let profile = crate::ARGS
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
    deserialize_with_env_expansion(conf) // Panics if configurations cannot be loaded
});

#[cfg(test)]
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(|| {
    seed_test_env_defaults();
    let conf = Config::builder()
        .add_source(config::File::with_name("config/profiles/unit-tests"))
        .set_override("profile", "unit-tests")
        .unwrap() // Add profile to the configuration
        .build()
        .unwrap();
    deserialize_with_env_expansion(conf) // Panics if configurations cannot be loaded
});

/// Environment variables referenced by `config/profiles/unit-tests.toml`
#[cfg(test)]
const TEST_ENV_DEFAULTS: &[(&str, &str)] = &[
    ("TRAVELRS_TEST_BOT_TOKEN", "MOCK_TOKEN"),
    ("TRAVELRS_TEST_DB_NAMESPACE", "test"),
    ("TRAVELRS_TEST_DB_NAME", "test"),
];

#[cfg(test)]
fn seed_test_env_defaults() {
    for (key, default) in TEST_ENV_DEFAULTS {
        if std::env::var(key).is_err() {
            // SAFETY: `SETTINGS` is a `LazyLock`, so this runs at most once,
            // before any test can observe the values. The keys are unique to
            // the unit-tests profile and are not mutated elsewhere.
            unsafe { std::env::set_var(key, default) };
        }
    }
}

/// Deserializes a [`Config`] into `T`, expanding `${VAR}` environment variable
/// references found in every string value (including those nested in tables
/// and arrays) before deserialization.
fn deserialize_with_env_expansion<T: for<'de> Deserialize<'de>>(config: Config) -> T {
    let map = config.collect().expect("Failed to collect config values");
    let root = Value::new(None, ValueKind::Table(map));
    let expanded = expand_env_in_value("", root);
    expanded
        .try_deserialize()
        .expect("Failed to deserialize expanded configuration")
}

/// Recursively expands `${VAR}` environment variable references in every
/// string leaf of `value`. Tables and arrays are traversed; other kinds are
/// left untouched.
fn expand_env_in_value(path: &str, value: Value) -> Value {
    let origin = value.origin().map(String::from);
    let kind = match value.kind {
        ValueKind::String(s) => ValueKind::String(expand_env_vars(&s, path)),
        ValueKind::Table(table) => {
            let expanded = table
                .into_iter()
                .map(|(k, v)| {
                    let child_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    let v = expand_env_in_value(&child_path, v);
                    (k, v)
                })
                .collect();
            ValueKind::Table(expanded)
        }
        ValueKind::Array(arr) => {
            let expanded = arr
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    let child_path = format!("{path}[{i}]");
                    expand_env_in_value(&child_path, v)
                })
                .collect();
            ValueKind::Array(expanded)
        }
        other => other,
    };
    Value::new(origin.as_ref(), kind)
}

/// Expands environment variable references in a string.
/// Supports `${VAR_NAME}` syntax. Unset variables are replaced with an empty string.
fn expand_env_vars(input: &str, property_name: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                var_name.push(c);
            }
            if let Ok(val) = std::env::var(&var_name) {
                result.push_str(&val)
            } else {
                panic!("Environment variable '{var_name}' for {property_name} should be set")
            }
        } else {
            result.push(ch);
        }
    }

    result
}

enum PropertySource {
    File,
    String,
}

impl PropertySource {
    fn from_str(s: &str) -> Self {
        match s {
            "file" => Self::File,
            "string" => Self::String,
            _ => panic!("Invalid property source: {s}. Expected 'file' or 'string'"),
        }
    }

    /// Resolves a property value from the specified source.
    ///
    /// # Arguments
    /// * `source` - The source type (File or String)
    /// * `value` - The path (for File) or the value itself (for String)
    /// * `property_name` - The name of the property for error messages (e.g., "Token", "Whitelist")
    fn resolve(source: PropertySource, value: &str, property_name: &str) -> String {
        match source {
            PropertySource::File => std::fs::read_to_string(value)
                .unwrap_or_else(|_| panic!("{property_name} file '{value}' should be readable")),
            PropertySource::String => value.to_string(),
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
    #[serde(default = "I18n::default_popular_currencies")]
    pub popular_currencies: Vec<String>,
}

impl I18n {
    fn default_popular_currencies() -> Vec<String> {
        ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "CNY"]
            .into_iter()
            .map(String::from)
            .collect()
    }
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
        let source = PropertySource::from_str(&self.bot.token_source);
        PropertySource::resolve(source, &self.bot.token.0, "Token")
    }

    pub fn chat_whitelist_value(&self) -> Vec<ChatId> {
        let Some(chat_whitelist_source) = &self.bot.chat_whitelist_source else {
            return Vec::new(); // No whitelist source specified, return empty vector
        };
        let Some(chat_whitelist) = &self.bot.chat_whitelist else {
            return Vec::new(); // No whitelist specified, return empty vector
        };

        let source = PropertySource::from_str(chat_whitelist_source);
        let content = PropertySource::resolve(source, chat_whitelist, "Whitelist");

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

#[cfg(test)]
mod tests {
    use super::*;

    // Small guard helper so tests don't leak env vars into each other.
    // Each test uses uniquely-named variables to remain parallel-safe.
    struct EnvGuard(&'static str);
    impl EnvGuard {
        fn set(name: &'static str, value: &str) -> Self {
            // SAFETY: tests use unique variable names per test to avoid races
            // with other tests mutating the same environment entry.
            unsafe { std::env::set_var(name, value) };
            Self(name)
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: see `EnvGuard::set`.
            unsafe { std::env::remove_var(self.0) };
        }
    }

    // ---------- expand_env_vars ----------

    #[test]
    fn expand_env_vars_returns_input_when_no_placeholders() {
        assert_eq!(expand_env_vars("plain string", "prop"), "plain string");
        assert_eq!(expand_env_vars("", "prop"), "");
    }

    #[test]
    fn expand_env_vars_substitutes_single_placeholder() {
        let _g = EnvGuard::set("TRAVELRS_TEST_SINGLE", "hello");
        assert_eq!(expand_env_vars("${TRAVELRS_TEST_SINGLE}", "prop"), "hello");
    }

    #[test]
    fn expand_env_vars_substitutes_multiple_placeholders() {
        let _a = EnvGuard::set("TRAVELRS_TEST_A", "foo");
        let _b = EnvGuard::set("TRAVELRS_TEST_B", "bar");
        assert_eq!(
            expand_env_vars("${TRAVELRS_TEST_A}-${TRAVELRS_TEST_B}", "prop"),
            "foo-bar"
        );
    }

    #[test]
    fn expand_env_vars_preserves_surrounding_literal_text() {
        let _g = EnvGuard::set("TRAVELRS_TEST_MIX", "middle");
        assert_eq!(
            expand_env_vars("start ${TRAVELRS_TEST_MIX} end", "prop"),
            "start middle end"
        );
    }

    #[test]
    fn expand_env_vars_leaves_lone_dollar_untouched() {
        // A '$' not followed by '{' is treated as a literal.
        assert_eq!(expand_env_vars("price: $5", "prop"), "price: $5");
        assert_eq!(expand_env_vars("$VAR", "prop"), "$VAR");
    }

    #[test]
    #[should_panic(expected = "Environment variable 'TRAVELRS_TEST_MISSING'")]
    fn expand_env_vars_panics_on_unset_variable() {
        // Ensure the variable is absent.
        // SAFETY: unique name; only this test manipulates it.
        unsafe { std::env::remove_var("TRAVELRS_TEST_MISSING") };
        let _ = expand_env_vars("${TRAVELRS_TEST_MISSING}", "prop");
    }

    // ---------- expand_env_in_value ----------

    #[test]
    fn expand_env_in_value_expands_string_leaf() {
        let _g = EnvGuard::set("TRAVELRS_TEST_LEAF", "expanded");
        let v = Value::new(None, ValueKind::String("${TRAVELRS_TEST_LEAF}".into()));
        let out = expand_env_in_value("bot.token", v);
        match out.kind {
            ValueKind::String(s) => assert_eq!(s, "expanded"),
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn expand_env_in_value_leaves_non_string_kinds_untouched() {
        let cases = [
            ValueKind::Boolean(true),
            ValueKind::I64(42),
            ValueKind::U64(7),
            ValueKind::Float(std::f64::consts::PI),
            ValueKind::Nil,
        ];
        for kind in cases {
            let v = Value::new(None, kind.clone());
            let out = expand_env_in_value("some.path", v);
            assert_eq!(out.kind, kind);
        }
    }

    #[test]
    fn expand_env_in_value_recurses_into_tables_and_arrays() {
        let _tok = EnvGuard::set("TRAVELRS_TEST_TOK", "TOKEN_XYZ");
        let _pwd = EnvGuard::set("TRAVELRS_TEST_PWD", "s3cret");
        let _cur = EnvGuard::set("TRAVELRS_TEST_CUR", "EUR");

        let mut inner = config::Map::<String, Value>::new();
        inner.insert(
            "token".into(),
            Value::new(None, ValueKind::String("${TRAVELRS_TEST_TOK}".into())),
        );
        inner.insert(
            "password".into(),
            Value::new(None, ValueKind::String("${TRAVELRS_TEST_PWD}".into())),
        );
        inner.insert("port".into(), Value::new(None, ValueKind::I64(8080)));

        let arr = ValueKind::Array(vec![
            Value::new(None, ValueKind::String("USD".into())),
            Value::new(None, ValueKind::String("${TRAVELRS_TEST_CUR}".into())),
        ]);
        inner.insert("currencies".into(), Value::new(None, arr));

        let root = Value::new(None, ValueKind::Table(inner));
        let expanded = expand_env_in_value("", root);

        let table = match expanded.kind {
            ValueKind::Table(t) => t,
            other => panic!("expected Table, got {other:?}"),
        };
        assert_eq!(
            table.get("token").unwrap().kind,
            ValueKind::String("TOKEN_XYZ".into())
        );
        assert_eq!(
            table.get("password").unwrap().kind,
            ValueKind::String("s3cret".into())
        );
        assert_eq!(table.get("port").unwrap().kind, ValueKind::I64(8080));

        let currencies = match &table.get("currencies").unwrap().kind {
            ValueKind::Array(a) => a.clone(),
            other => panic!("expected Array, got {other:?}"),
        };
        assert_eq!(currencies[0].kind, ValueKind::String("USD".into()));
        assert_eq!(currencies[1].kind, ValueKind::String("EUR".into()));
    }

    // ---------- deserialize_with_env_expansion (end-to-end) ----------

    #[derive(Debug, Deserialize, PartialEq)]
    struct DbFixture {
        address: String,
        password: String,
        port: i64,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct RootFixture {
        name: String,
        database: DbFixture,
        currencies: Vec<String>,
    }

    #[test]
    fn deserialize_with_env_expansion_substitutes_across_the_whole_tree() {
        let _host = EnvGuard::set("TRAVELRS_TEST_HOST", "db.example.com");
        let _pwd = EnvGuard::set("TRAVELRS_TEST_E2E_PWD", "hunter2");
        let _cur = EnvGuard::set("TRAVELRS_TEST_E2E_CUR", "JPY");

        let toml = r#"
name = "travelrs"
currencies = ["USD", "${TRAVELRS_TEST_E2E_CUR}"]

[database]
address = "wss://${TRAVELRS_TEST_HOST}:8000"
password = "${TRAVELRS_TEST_E2E_PWD}"
port = 8000
"#;

        let cfg = Config::builder()
            .add_source(config::File::from_str(toml, config::FileFormat::Toml))
            .build()
            .expect("build config");

        let out: RootFixture = deserialize_with_env_expansion(cfg);

        assert_eq!(
            out,
            RootFixture {
                name: "travelrs".into(),
                database: DbFixture {
                    address: "wss://db.example.com:8000".into(),
                    password: "hunter2".into(),
                    port: 8000,
                },
                currencies: vec!["USD".into(), "JPY".into()],
            }
        );
    }

    // ---------- unit-tests profile (real file, real static) ----------

    #[test]
    fn unit_tests_profile_expands_env_var_defaults() {
        // Force the default seeding path by making sure the vars aren't set
        // ahead of the first `SETTINGS` access. Safe because the values are
        // only referenced through the unit-tests profile.
        for (key, _) in TEST_ENV_DEFAULTS {
            // SAFETY: see `seed_test_env_defaults`; keys are exclusive to this profile.
            unsafe { std::env::remove_var(key) };
        }

        // Touching `SETTINGS` triggers `seed_test_env_defaults` (only on the
        // very first access process-wide) followed by env var expansion of
        // the placeholders declared in `config/profiles/unit-tests.toml`.
        let settings = &*SETTINGS;

        assert_eq!(settings.profile, "unit-tests");
        assert_eq!(settings.bot.token.0, "MOCK_TOKEN");
        assert_eq!(settings.database.namespace, "test");
        assert_eq!(settings.database.database, "test");

        // No placeholder should have survived expansion.
        assert!(!settings.bot.token.0.contains("${"));
        assert!(!settings.database.namespace.contains("${"));
        assert!(!settings.database.database.contains("${"));
    }
}
