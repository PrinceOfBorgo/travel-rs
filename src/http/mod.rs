mod routes;
mod session;
mod trip;
mod validate;

use axum::Router;
use session::SessionState;
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any};
use tower_http::cors::{Any as CorsAny, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use trip::TripState;

use crate::settings::SETTINGS;

/// Start the HTTP server. Always binds to the configured address.
/// If no `[api]` section is present, uses a default bind address.
pub async fn start_http(bot_token: String, db: Arc<Surreal<Any>>) {
    let bind = SETTINGS
        .api
        .as_ref()
        .map(|cfg| cfg.bind.as_str())
        .unwrap_or("0.0.0.0:8080");

    let shared_token = Arc::new(bot_token);
    let session_state = SessionState {
        bot_token: shared_token.clone(),
    };
    let trip_state = TripState {
        bot_token: shared_token.clone(),
        db,
    };

    let cors = CorsLayer::new()
        .allow_origin(CorsAny)
        .allow_methods(CorsAny)
        .allow_headers(CorsAny);

    // Core routes: healthz, version
    let core_routes = Router::new()
        .route("/healthz", axum::routing::get(routes::healthz))
        .route("/api/version", axum::routing::get(routes::version));

    // API routes requiring bot token (e.g., miniapp auth)
    let api_routes = Router::new()
        .route(
            "/api/miniapp/auth",
            axum::routing::post(validate::validate_init_data),
        )
        .with_state(shared_token);

    // Session auth route
    let session_routes = Router::new()
        .route(
            "/api/auth/session",
            axum::routing::post(session::create_session),
        )
        .with_state(session_state);

    // Trip API routes (read-only, authenticated via X-Init-Data)
    let trip_api_routes = trip::trip_routes(trip_state);

    // Optional miniapp static file serving with SPA fallback
    let app = if let Some(api_cfg) = &SETTINGS.api {
        let miniapp_dir = PathBuf::from(&api_cfg.miniapp_dir);
        let index_path = miniapp_dir.join("index.html");

        // ServeDir with SPA fallback: unknown paths serve index.html
        let miniapp_service =
            ServeDir::new(&miniapp_dir).not_found_service(ServeFile::new(&index_path));

        Router::new()
            .nest_service("/miniapp", miniapp_service)
            .merge(core_routes)
            .merge(api_routes)
            .merge(session_routes)
            .merge(trip_api_routes)
            .layer(cors)
    } else {
        Router::new()
            .merge(core_routes)
            .merge(api_routes)
            .merge(session_routes)
            .merge(trip_api_routes)
            .layer(cors)
    };

    tracing::info!("Starting HTTP server on {bind}");
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .expect("failed to bind HTTP listener");
    axum::serve(listener, app).await.expect("HTTP server error");
}
