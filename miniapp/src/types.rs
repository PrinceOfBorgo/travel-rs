//! Shared API response types, mirroring the server-side JSON shapes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct SessionResponse {
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SummaryResponse {
    pub traveler_count: usize,
    pub expense_count: usize,
    pub transfer_count: usize,
    pub total_expenses: String,
    pub currency: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SettingsResponse {
    pub currency: String,
    pub language: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TravelerResponse {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExpenseResponse {
    pub number: i64,
    pub description: String,
    pub amount: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransferResponse {
    pub number: i64,
    pub amount: String,
    pub sender_name: String,
    pub receiver_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BalanceResponse {
    pub debt: String,
    pub debtor_name: String,
    pub creditor_name: String,
}

/// Generic mutation response.
#[derive(Clone, Debug, Deserialize)]
pub struct MutationResponse {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
}

// ── Request types ──

#[derive(Serialize)]
pub struct AddTravelerRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct DeleteTravelerRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct AddExpenseRequest {
    pub description: String,
    pub amount: String,
    pub paid_by: String,
    pub shares: std::collections::BTreeMap<String, String>,
}

#[derive(Serialize)]
pub struct DeleteExpenseRequest {
    pub number: i64,
}

#[derive(Serialize)]
pub struct AddTransferRequest {
    pub from: String,
    pub to: String,
    pub amount: String,
}

#[derive(Serialize)]
pub struct DeleteTransferRequest {
    pub number: i64,
}

#[derive(Serialize)]
pub struct SetCurrencyRequest {
    pub currency: String,
}

#[derive(Serialize)]
pub struct SetLanguageRequest {
    pub language: String,
}
