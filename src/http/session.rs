//! Session creation and membership verification for Mini App auth.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatId, ChatMemberKind, UserId};

use super::validate::verify_and_parse_init_data;

/// State shared with session endpoints.
#[derive(Clone)]
pub struct SessionState {
    pub bot_token: Arc<String>,
}

#[derive(Deserialize)]
pub struct SessionRequest {
    /// The raw initData string from Telegram.WebApp.initData
    pub init_data: String,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub user_id: i64,
    pub chat_id: Option<i64>,
    pub is_member: bool,
    /// Unix timestamp when auth was created
    pub auth_date: u64,
}

/// POST /api/auth/session
///
/// Creates a session by:
/// 1. Validating initData hash (rejects invalid signatures)
/// 2. Checking auth_date freshness (rejects stale auth > 5 minutes)
/// 3. Extracting chat_id from start_param (if present)
/// 4. Verifying chat membership via getChatMember (if chat_id present)
pub async fn create_session(
    State(state): State<SessionState>,
    Json(payload): Json<SessionRequest>,
) -> (StatusCode, Json<SessionResponse>) {
    // Step 1 & 2: Validate initData and check auth_date
    let validated = match verify_and_parse_init_data(&payload.init_data, &state.bot_token) {
        Ok(v) => v,
        Err(msg) => {
            tracing::warn!("Session creation failed: {msg}");
            return (
                StatusCode::UNAUTHORIZED,
                Json(SessionResponse {
                    ok: false,
                    session: None,
                    error: Some(msg),
                }),
            );
        }
    };

    // Step 3 & 4: Check membership if chat_id is present
    let is_member = match validated.chat_id {
        Some(chat_id) => {
            match check_chat_membership(&state.bot_token, chat_id, validated.user_id).await {
                Ok(member) => member,
                Err(msg) => {
                    tracing::warn!(
                        "Membership check failed for user {} in chat {}: {msg}",
                        validated.user_id,
                        chat_id
                    );
                    return (
                        StatusCode::FORBIDDEN,
                        Json(SessionResponse {
                            ok: false,
                            session: None,
                            error: Some(msg),
                        }),
                    );
                }
            }
        }
        None => {
            // No chat_id in start_param, session is valid but not tied to a chat
            true
        }
    };

    if !is_member {
        return (
            StatusCode::FORBIDDEN,
            Json(SessionResponse {
                ok: false,
                session: None,
                error: Some("User is not a member of the chat".into()),
            }),
        );
    }

    tracing::info!(
        "Session created for user {} in chat {:?}",
        validated.user_id,
        validated.chat_id
    );

    (
        StatusCode::OK,
        Json(SessionResponse {
            ok: true,
            session: Some(SessionInfo {
                user_id: validated.user_id,
                chat_id: validated.chat_id,
                is_member,
                auth_date: validated.auth_date,
            }),
            error: None,
        }),
    )
}

/// Checks if a user is a member of a chat using Telegram's getChatMember API.
/// Returns Ok(true) if the user is a member (or admin/creator), Ok(false) if not.
/// Returns Err if the API call fails or user is kicked/left.
async fn check_chat_membership(
    bot_token: &str,
    chat_id: i64,
    user_id: i64,
) -> Result<bool, String> {
    let bot = Bot::new(bot_token);

    let member = bot
        .get_chat_member(ChatId(chat_id), UserId(user_id as u64))
        .await
        .map_err(|e| format!("Failed to check membership: {e}"))?;

    // Check if user is an active member of the chat
    let is_active_member = match &member.kind {
        ChatMemberKind::Owner(_) | ChatMemberKind::Administrator(_) | ChatMemberKind::Member(_) => {
            true
        }
        ChatMemberKind::Restricted(r) => r.is_member,
        _ => false,
    };

    Ok(is_active_member)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            user_id: 12345,
            chat_id: Some(-100123456),
            is_member: true,
            auth_date: 1700000000,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("12345"));
        assert!(json.contains("-100123456"));
    }
}
