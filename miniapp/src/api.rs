//! HTTP client for the trip API.

use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use crate::types::*;

async fn fetch_json<T: DeserializeOwned>(url: &str, init_data: &str) -> Result<T, String> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    request
        .headers()
        .set("X-Init-Data", init_data)
        .map_err(|e| format!("Failed to set header: {e:?}"))?;

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response cast failed".to_string())?;

    if !resp.ok() {
        let status = resp.status();
        let body = JsFuture::from(resp.text().map_err(|_| format!("HTTP {status}"))?)
            .await
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    let text = JsFuture::from(resp.text().map_err(|e| format!("text() failed: {e:?}"))?)
        .await
        .map_err(|e| format!("text() await failed: {e:?}"))?;

    let text_str = text.as_string().ok_or("Response is not a string")?;
    serde_json::from_str(&text_str).map_err(|e| format!("Parse failed: {e}"))
}

async fn post_json<Req: serde::Serialize, Resp: DeserializeOwned>(
    url: &str,
    body: &Req,
) -> Result<Resp, String> {
    send_json("POST", url, body, None).await
}

async fn send_json_auth<Req: serde::Serialize, Resp: DeserializeOwned>(
    method: &str,
    url: &str,
    body: &Req,
    init_data: &str,
) -> Result<Resp, String> {
    send_json(method, url, body, Some(init_data)).await
}

async fn send_json<Req: serde::Serialize, Resp: DeserializeOwned>(
    method: &str,
    url: &str,
    body: &Req,
    init_data: Option<&str>,
) -> Result<Resp, String> {
    let opts = RequestInit::new();
    opts.set_method(method);

    let json_body =
        serde_json::to_string(body).map_err(|e| format!("Serialize failed: {e}"))?;
    opts.set_body(&wasm_bindgen::JsValue::from_str(&json_body));

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {e:?}"))?;

    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set header: {e:?}"))?;

    if let Some(data) = init_data {
        request
            .headers()
            .set("X-Init-Data", data)
            .map_err(|e| format!("Failed to set auth header: {e:?}"))?;
    }

    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {e:?}"))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response cast failed".to_string())?;

    let text = JsFuture::from(resp.text().map_err(|e| format!("text() failed: {e:?}"))?)
        .await
        .map_err(|e| format!("text() await failed: {e:?}"))?;

    let text_str = text.as_string().ok_or("Response is not a string")?;

    if !resp.ok() {
        // Try to parse error body
        if let Ok(err) = serde_json::from_str::<MutationResponse>(&text_str) {
            return Err(err.error.unwrap_or(format!("HTTP {}", resp.status())));
        }
        return Err(format!("HTTP {}: {text_str}", resp.status()));
    }

    serde_json::from_str(&text_str).map_err(|e| format!("Parse failed: {e}"))
}

// ── Session ──

pub async fn create_session(init_data: &str) -> Result<SessionResponse, String> {
    #[derive(serde::Serialize)]
    struct Req {
        init_data: String,
    }
    post_json("/api/auth/session", &Req { init_data: init_data.to_owned() }).await
}

// ── Read endpoints ──

pub async fn fetch_summary(init_data: &str) -> Result<SummaryResponse, String> {
    fetch_json("/api/trip/summary", init_data).await
}

pub async fn fetch_settings(init_data: &str) -> Result<SettingsResponse, String> {
    fetch_json("/api/trip/settings", init_data).await
}

pub async fn fetch_travelers(init_data: &str) -> Result<Vec<TravelerResponse>, String> {
    fetch_json("/api/trip/travelers", init_data).await
}

pub async fn fetch_expenses(init_data: &str) -> Result<Vec<ExpenseResponse>, String> {
    fetch_json("/api/trip/expenses", init_data).await
}

pub async fn fetch_transfers(init_data: &str) -> Result<Vec<TransferResponse>, String> {
    fetch_json("/api/trip/transfers", init_data).await
}

pub async fn fetch_balances(init_data: &str) -> Result<Vec<BalanceResponse>, String> {
    fetch_json("/api/trip/balances", init_data).await
}

// ── Mutation endpoints ──

pub async fn add_traveler(init_data: &str, name: &str) -> Result<MutationResponse, String> {
    send_json_auth(
        "POST",
        "/api/trip/travelers",
        &AddTravelerRequest { name: name.to_owned() },
        init_data,
    )
    .await
}

pub async fn delete_traveler(init_data: &str, name: &str) -> Result<MutationResponse, String> {
    send_json_auth(
        "DELETE",
        "/api/trip/travelers",
        &DeleteTravelerRequest { name: name.to_owned() },
        init_data,
    )
    .await
}

pub async fn add_expense(
    init_data: &str,
    req: &AddExpenseRequest,
) -> Result<MutationResponse, String> {
    send_json_auth("POST", "/api/trip/expenses", req, init_data).await
}

pub async fn delete_expense(init_data: &str, number: i64) -> Result<MutationResponse, String> {
    send_json_auth(
        "DELETE",
        "/api/trip/expenses",
        &DeleteExpenseRequest { number },
        init_data,
    )
    .await
}

pub async fn add_transfer(
    init_data: &str,
    from: &str,
    to: &str,
    amount: &str,
) -> Result<MutationResponse, String> {
    send_json_auth(
        "POST",
        "/api/trip/transfers",
        &AddTransferRequest {
            from: from.to_owned(),
            to: to.to_owned(),
            amount: amount.to_owned(),
        },
        init_data,
    )
    .await
}

pub async fn delete_transfer(init_data: &str, number: i64) -> Result<MutationResponse, String> {
    send_json_auth(
        "DELETE",
        "/api/trip/transfers",
        &DeleteTransferRequest { number },
        init_data,
    )
    .await
}

pub async fn set_currency(
    init_data: &str,
    currency: &str,
) -> Result<MutationResponse, String> {
    send_json_auth(
        "PUT",
        "/api/trip/settings/currency",
        &SetCurrencyRequest { currency: currency.to_owned() },
        init_data,
    )
    .await
}

pub async fn set_language(
    init_data: &str,
    language: &str,
) -> Result<MutationResponse, String> {
    send_json_auth(
        "PUT",
        "/api/trip/settings/language",
        &SetLanguageRequest { language: language.to_owned() },
        init_data,
    )
    .await
}
