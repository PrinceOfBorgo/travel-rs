use axum::Json;
use serde::Serialize;

/// Health check endpoint.
/// Returns 200 OK with "ok" body.
pub async fn healthz() -> &'static str {
    "ok"
}

#[derive(Serialize)]
pub struct VersionInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Version endpoint.
/// Returns JSON with package name and version.
pub async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}
