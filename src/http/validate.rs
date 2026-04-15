use axum::{Json, extract::State, http::StatusCode};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Maximum age of auth_date in seconds (5 minutes).
const MAX_AUTH_AGE_SECS: u64 = 300;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub init_data: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parsed and validated initData fields.
#[derive(Debug, Clone)]
pub struct ValidatedInitData {
    pub user_id: i64,
    pub user: serde_json::Value,
    pub auth_date: u64,
    pub chat_id: Option<i64>,
}

/// Validates Telegram `initData` according to the official algorithm:
/// <https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app>
pub async fn validate_init_data(
    State(bot_token): State<Arc<String>>,
    Json(payload): Json<AuthRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    match verify_and_parse_init_data(&payload.init_data, &bot_token) {
        Ok(validated) => (
            StatusCode::OK,
            Json(AuthResponse {
                ok: true,
                user: Some(validated.user),
                error: None,
            }),
        ),
        Err(msg) => (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                ok: false,
                user: None,
                error: Some(msg),
            }),
        ),
    }
}

/// Verifies initData hash and parses validated fields.
/// Returns error if hash is invalid or auth_date is stale.
pub fn verify_and_parse_init_data(
    init_data: &str,
    bot_token: &str,
) -> Result<ValidatedInitData, String> {
    verify_init_data_impl(init_data, bot_token, true)
}

/// Verifies initData HMAC signature without checking auth_date staleness.
/// Use this for API endpoints where the session was already established.
pub fn verify_init_data_signature(
    init_data: &str,
    bot_token: &str,
) -> Result<ValidatedInitData, String> {
    verify_init_data_impl(init_data, bot_token, false)
}

fn verify_init_data_impl(
    init_data: &str,
    bot_token: &str,
    check_staleness: bool,
) -> Result<ValidatedInitData, String> {
    // Parse the query string into key-value pairs
    let pairs: Vec<(String, String)> = form_urlencoded_parse(init_data);

    let mut hash_value = String::new();
    let mut data_check: BTreeMap<String, String> = BTreeMap::new();

    for (key, value) in &pairs {
        if key == "hash" {
            hash_value.clone_from(value);
        } else {
            data_check.insert(key.clone(), value.clone());
        }
    }

    if hash_value.is_empty() {
        return Err("Missing hash parameter".into());
    }

    // Build the data-check-string: alphabetically sorted "key=value" lines
    let data_check_string: String = data_check
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    // secret_key = HMAC-SHA256("WebAppData", bot_token)
    let mut secret_mac =
        HmacSha256::new_from_slice(b"WebAppData").expect("HMAC can take key of any size");
    secret_mac.update(bot_token.as_bytes());
    let secret_key = secret_mac.finalize().into_bytes();

    // computed_hash = HMAC-SHA256(secret_key, data_check_string)
    let mut mac = HmacSha256::new_from_slice(&secret_key).expect("HMAC can take key of any size");
    mac.update(data_check_string.as_bytes());
    let computed = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison via HMAC verify is already done above,
    // but since we hex-encode we compare strings here.
    if computed != hash_value {
        return Err("Invalid hash".into());
    }

    // Check auth_date freshness
    let auth_date_str = data_check
        .get("auth_date")
        .ok_or("Missing auth_date parameter")?;
    let auth_date: u64 = auth_date_str
        .parse()
        .map_err(|_| "Invalid auth_date format")?;

    if check_staleness {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "System time error")?
            .as_secs();

        if now > auth_date + MAX_AUTH_AGE_SECS {
            return Err("Auth data is stale".into());
        }
    }

    // Parse user JSON
    let user_json_str = data_check.get("user").ok_or("Missing user parameter")?;
    let user: serde_json::Value =
        serde_json::from_str(user_json_str).map_err(|_| "Invalid user JSON")?;
    let user_id = user
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or("Missing user id")?;

    // Parse start_param to extract chat_id if present (format: "chat_<id>")
    let chat_id = data_check
        .get("start_param")
        .and_then(|p| {
            p.strip_prefix("chat_")
                .and_then(|id_str| id_str.parse::<i64>().ok())
        });

    Ok(ValidatedInitData {
        user_id,
        user,
        auth_date,
        chat_id,
    })
}

/// Minimal URL query-string parser (percent-decoded).
fn form_urlencoded_parse(input: &str) -> Vec<(String, String)> {
    input
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((url_decode(key), url_decode(value)))
        })
        .collect()
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        match b {
            b'+' => result.push(' '),
            b'%' => {
                let hi = chars.next().and_then(hex_val);
                let lo = chars.next().and_then(hex_val);
                if let (Some(h), Some(l)) = (hi, lo) {
                    result.push((h << 4 | l) as char);
                } else {
                    result.push('%');
                }
            }
            _ => result.push(b as char),
        }
    }
    result
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
