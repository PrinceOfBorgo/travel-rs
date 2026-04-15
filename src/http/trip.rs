//! Trip API endpoints (read-only + mutations).
//!
//! All endpoints require a valid Telegram `initData` in the `X-Init-Data` header.
//! The HMAC signature is verified but staleness is not enforced (it was checked
//! during session creation).

use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::{StatusCode, request::Parts},
    routing::{delete, get, post, put},
    Router,
};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};
use teloxide::types::ChatId;

use crate::balance::Balance;
use crate::services::{self, ServiceError};
use crate::services::expense::AmountEnum;
use crate::tables::chat::Chat;
use crate::tables::expense::Expense;
use crate::tables::traveler::{Name, Traveler};
use crate::transfer::Transfer;

use super::validate::verify_init_data_signature;

// ── State ──

#[derive(Clone)]
pub struct TripState {
    pub bot_token: Arc<String>,
    pub db: Arc<Surreal<Any>>,
}

impl FromRef<TripState> for Arc<String> {
    fn from_ref(state: &TripState) -> Self {
        state.bot_token.clone()
    }
}

impl FromRef<TripState> for Arc<Surreal<Any>> {
    fn from_ref(state: &TripState) -> Self {
        state.db.clone()
    }
}

// ── Auth extractor ──

/// Extractor that validates the `X-Init-Data` header and provides the authenticated chat_id.
pub struct Auth {
    pub chat_id: i64,
    #[allow(dead_code)]
    pub user_id: i64,
}

#[derive(Serialize)]
pub struct ErrorBody {
    ok: bool,
    error: String,
}

impl<S> FromRequestParts<S> for Auth
where
    TripState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let trip_state = TripState::from_ref(state);

        let init_data = parts
            .headers
            .get("x-init-data")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorBody {
                        ok: false,
                        error: "Missing X-Init-Data header".into(),
                    }),
                )
            })?;

        let validated =
            verify_init_data_signature(init_data, &trip_state.bot_token).map_err(|msg| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorBody {
                        ok: false,
                        error: msg,
                    }),
                )
            })?;

        let chat_id = validated.chat_id.ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    ok: false,
                    error: "No chat_id in initData".into(),
                }),
            )
        })?;

        Ok(Auth {
            chat_id,
            user_id: validated.user_id,
        })
    }
}

// ── API response types ──

#[derive(Serialize)]
struct SettingsResponse {
    currency: String,
    language: String,
}

#[derive(Serialize)]
struct TravelerResponse {
    name: String,
}

#[derive(Serialize)]
struct ExpenseResponse {
    number: i64,
    description: String,
    amount: String,
    timestamp_utc: String,
}

#[derive(Serialize)]
struct TransferResponse {
    number: i64,
    amount: String,
    sender_name: String,
    receiver_name: String,
    timestamp_utc: String,
}

#[derive(Serialize)]
struct BalanceResponse {
    debt: String,
    debtor_name: String,
    creditor_name: String,
}

#[derive(Serialize)]
struct SummaryResponse {
    traveler_count: usize,
    expense_count: usize,
    transfer_count: usize,
    total_expenses: String,
    currency: String,
}

// ── Router ──

pub fn trip_routes(state: TripState) -> Router {
    Router::new()
        // Read endpoints
        .route("/api/trip/settings", get(get_settings))
        .route("/api/trip/travelers", get(get_travelers))
        .route("/api/trip/expenses", get(get_expenses))
        .route("/api/trip/transfers", get(get_transfers))
        .route("/api/trip/balances", get(get_balances))
        .route("/api/trip/summary", get(get_summary))
        // Mutation endpoints
        .route("/api/trip/travelers", post(add_traveler))
        .route("/api/trip/travelers", delete(remove_traveler))
        .route("/api/trip/expenses", post(add_expense))
        .route("/api/trip/expenses", delete(remove_expense))
        .route("/api/trip/transfers", post(add_transfer))
        .route("/api/trip/transfers", delete(remove_transfer))
        .route("/api/trip/settings/currency", put(set_currency))
        .route("/api/trip/settings/language", put(set_language))
        .with_state(state)
}

// ── Mutation request types ──

#[derive(Deserialize)]
struct AddTravelerRequest {
    name: String,
}

#[derive(Deserialize)]
struct DeleteTravelerRequest {
    name: String,
}

#[derive(Deserialize)]
struct AddExpenseRequest {
    description: String,
    amount: String,
    paid_by: String,
    /// Map of traveler name → share. Share can be a fixed amount ("50"),
    /// a percentage ("25%"), or "equal" for dynamic/even split.
    shares: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct DeleteExpenseRequest {
    number: i64,
}

#[derive(Deserialize)]
struct AddTransferRequest {
    from: String,
    to: String,
    amount: String,
}

#[derive(Deserialize)]
struct DeleteTransferRequest {
    number: i64,
}

#[derive(Deserialize)]
struct SetCurrencyRequest {
    currency: String,
}

#[derive(Deserialize)]
struct SetLanguageRequest {
    language: String,
}

/// Generic success response for mutations.
#[derive(Serialize)]
struct MutationOk {
    ok: bool,
}

// ── Handlers ──

async fn get_settings(
    auth: Auth,
    State(state): State<TripState>,
) -> Result<Json<SettingsResponse>, (StatusCode, Json<ErrorBody>)> {
    let chat = Chat::db_select_by_id(state.db, ChatId(auth.chat_id))
        .await
        .map_err(db_error)?
        .ok_or_else(|| not_found("Chat not found"))?;

    Ok(Json(SettingsResponse {
        currency: chat.currency,
        language: chat.lang,
    }))
}

async fn get_travelers(
    auth: Auth,
    State(state): State<TripState>,
) -> Result<Json<Vec<TravelerResponse>>, (StatusCode, Json<ErrorBody>)> {
    let travelers = Traveler::db_select(state.db, ChatId(auth.chat_id))
        .await
        .map_err(db_error)?;

    Ok(Json(
        travelers
            .into_iter()
            .map(|t| TravelerResponse {
                name: t.name.to_string(),
            })
            .collect(),
    ))
}

async fn get_expenses(
    auth: Auth,
    State(state): State<TripState>,
) -> Result<Json<Vec<ExpenseResponse>>, (StatusCode, Json<ErrorBody>)> {
    let expenses = Expense::db_select(state.db, ChatId(auth.chat_id))
        .await
        .map_err(db_error)?;

    Ok(Json(
        expenses
            .into_iter()
            .map(|e| ExpenseResponse {
                number: e.number,
                description: e.description,
                amount: e.amount.to_string(),
                timestamp_utc: e.timestamp_utc.to_string(),
            })
            .collect(),
    ))
}

async fn get_transfers(
    auth: Auth,
    State(state): State<TripState>,
) -> Result<Json<Vec<TransferResponse>>, (StatusCode, Json<ErrorBody>)> {
    let transfers = Transfer::transfers(state.db, ChatId(auth.chat_id))
        .await
        .map_err(db_error)?;

    Ok(Json(
        transfers
            .into_iter()
            .map(|t| TransferResponse {
                number: t.number,
                amount: t.amount.to_string(),
                sender_name: t.sender_name.to_string(),
                receiver_name: t.receiver_name.to_string(),
                timestamp_utc: t.timestamp_utc.to_string(),
            })
            .collect(),
    ))
}

async fn get_balances(
    auth: Auth,
    State(state): State<TripState>,
) -> Result<Json<Vec<BalanceResponse>>, (StatusCode, Json<ErrorBody>)> {
    let balances = Balance::balances(state.db, ChatId(auth.chat_id))
        .await
        .map_err(db_error)?;

    Ok(Json(
        balances
            .into_iter()
            .map(|b| BalanceResponse {
                debt: b.debt.to_string(),
                debtor_name: b.debtor_name.to_string(),
                creditor_name: b.creditor_name.to_string(),
            })
            .collect(),
    ))
}

async fn get_summary(
    auth: Auth,
    State(state): State<TripState>,
) -> Result<Json<SummaryResponse>, (StatusCode, Json<ErrorBody>)> {
    let chat_id = ChatId(auth.chat_id);
    let db = &state.db;

    let chat = Chat::db_select_by_id(db.clone(), chat_id)
        .await
        .map_err(db_error)?
        .ok_or_else(|| not_found("Chat not found"))?;

    let travelers = Traveler::db_select(db.clone(), chat_id)
        .await
        .map_err(db_error)?;

    let expenses = Expense::db_select(db.clone(), chat_id)
        .await
        .map_err(db_error)?;

    let transfers = Transfer::transfers(db.clone(), chat_id)
        .await
        .map_err(db_error)?;

    let total: Decimal = expenses.iter().map(|e| e.amount).sum();

    Ok(Json(SummaryResponse {
        traveler_count: travelers.len(),
        expense_count: expenses.len(),
        transfer_count: transfers.len(),
        total_expenses: total.to_string(),
        currency: chat.currency,
    }))
}

// ── Mutation handlers ──

async fn add_traveler(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<AddTravelerRequest>,
) -> ApiResult<Json<MutationOk>> {
    let name =
        Name::from_str(payload.name.trim()).map_err(|e| bad_request(&format!("Invalid name: {e}")))?;
    if name.is_empty() {
        return Err(bad_request("Name must not be empty"));
    }

    services::traveler::add_traveler(state.db, ChatId(auth.chat_id), &name)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn remove_traveler(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<DeleteTravelerRequest>,
) -> ApiResult<Json<MutationOk>> {
    let name =
        Name::from_str(payload.name.trim()).map_err(|e| bad_request(&format!("Invalid name: {e}")))?;

    services::traveler::delete_traveler(state.db, ChatId(auth.chat_id), &name)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn add_expense(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<AddExpenseRequest>,
) -> ApiResult<Json<MutationOk>> {
    let chat_id = ChatId(auth.chat_id);
    let description = payload.description.trim().to_owned();
    if description.is_empty() {
        return Err(bad_request("Description must not be empty"));
    }

    let amount = Decimal::from_str(&payload.amount)
        .map_err(|_| bad_request("Invalid amount"))?;
    if amount <= Decimal::ZERO {
        return Err(bad_request("Amount must be positive"));
    }

    let paid_by_name = Name::from_str(payload.paid_by.trim())
        .map_err(|e| bad_request(&format!("Invalid payer name: {e}")))?;
    let paid_by = Traveler::db_select_by_name(state.db.clone(), chat_id, &paid_by_name)
        .await
        .map_err(db_error)?
        .ok_or_else(|| not_found("Payer not found"))?;

    if payload.shares.is_empty() {
        return Err(bad_request("At least one share is required"));
    }

    // Parse and validate shares
    let mut parsed_shares: BTreeMap<Name, AmountEnum> = BTreeMap::new();
    for (name_str, share_str) in &payload.shares {
        let name = Name::from_str(name_str.trim())
            .map_err(|e| bad_request(&format!("Invalid traveler name '{name_str}': {e}")))?;

        // Verify traveler exists
        Traveler::db_select_by_name(state.db.clone(), chat_id, &name)
            .await
            .map_err(db_error)?
            .ok_or_else(|| not_found(&format!("Traveler '{name}' not found")))?;

        let kind = if share_str == "equal" {
            AmountEnum::Dynamic
        } else if let Some(pct) = share_str.strip_suffix('%') {
            let pct_val = Decimal::from_str(pct.trim())
                .map_err(|_| bad_request(&format!("Invalid percentage '{share_str}'")))?;
            AmountEnum::Percentage(pct_val)
        } else {
            let fixed = Decimal::from_str(share_str.trim())
                .map_err(|_| bad_request(&format!("Invalid share amount '{share_str}'")))?;
            AmountEnum::Fixed(fixed)
        };
        parsed_shares.insert(name, kind);
    }

    // Compute final decimal shares
    let shares = services::expense::compute_shares(amount, parsed_shares)
        .map_err(|e| bad_request(&e.to_string()))?;

    // Create expense with share relationships and debt update
    services::expense::create_expense(state.db, chat_id, description, amount, &paid_by, shares)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn remove_expense(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<DeleteExpenseRequest>,
) -> ApiResult<Json<MutationOk>> {
    services::expense::delete_expense(state.db, ChatId(auth.chat_id), payload.number)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn add_transfer(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<AddTransferRequest>,
) -> ApiResult<Json<MutationOk>> {
    let from_name = Name::from_str(payload.from.trim())
        .map_err(|e| bad_request(&format!("Invalid sender name: {e}")))?;
    let to_name = Name::from_str(payload.to.trim())
        .map_err(|e| bad_request(&format!("Invalid receiver name: {e}")))?;
    let amount = Decimal::from_str(&payload.amount)
        .map_err(|_| bad_request("Invalid amount"))?;
    if amount <= Decimal::ZERO {
        return Err(bad_request("Amount must be positive"));
    }

    services::transfer::create_transfer(state.db, ChatId(auth.chat_id), &from_name, &to_name, amount)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn remove_transfer(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<DeleteTransferRequest>,
) -> ApiResult<Json<MutationOk>> {
    services::transfer::delete_transfer(state.db, ChatId(auth.chat_id), payload.number)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn set_currency(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<SetCurrencyRequest>,
) -> ApiResult<Json<MutationOk>> {
    services::settings::set_currency(state.db, ChatId(auth.chat_id), &payload.currency)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

async fn set_language(
    auth: Auth,
    State(state): State<TripState>,
    Json(payload): Json<SetLanguageRequest>,
) -> ApiResult<Json<MutationOk>> {
    use unic_langid::LanguageIdentifier;

    let langid: LanguageIdentifier = payload
        .language
        .trim()
        .parse()
        .map_err(|_| bad_request("Invalid language identifier"))?;

    services::settings::set_language(state.db, ChatId(auth.chat_id), &langid)
        .await
        .map_err(service_error)?;

    Ok(Json(MutationOk { ok: true }))
}

// ── Helpers ──

type ApiResult<T> = Result<T, (StatusCode, Json<ErrorBody>)>;

fn service_error(err: ServiceError) -> (StatusCode, Json<ErrorBody>) {
    match &err {
        ServiceError::AlreadyExists(_) => conflict(&err.to_string()),
        ServiceError::NotFound(_) => not_found(&err.to_string()),
        ServiceError::HasAssociatedExpenses(_) => conflict(&err.to_string()),
        ServiceError::EmptyInput(_) => bad_request(&err.to_string()),
        ServiceError::ShareComputation(_) => bad_request(&err.to_string()),
        ServiceError::NoExpenseCreated => db_error_msg(&err.to_string()),
        ServiceError::LanguageNotAvailable { .. } => bad_request(&err.to_string()),
        ServiceError::Database(e) => {
            tracing::error!("Database error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    ok: false,
                    error: "Internal server error".into(),
                }),
            )
        }
    }
}

fn db_error(err: surrealdb::Error) -> (StatusCode, Json<ErrorBody>) {
    tracing::error!("Database error: {err}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            ok: false,
            error: "Internal server error".into(),
        }),
    )
}

fn not_found(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody {
            ok: false,
            error: msg.into(),
        }),
    )
}

fn bad_request(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorBody {
            ok: false,
            error: msg.into(),
        }),
    )
}

fn conflict(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorBody {
            ok: false,
            error: msg.into(),
        }),
    )
}

fn db_error_msg(msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            ok: false,
            error: msg.into(),
        }),
    )
}
