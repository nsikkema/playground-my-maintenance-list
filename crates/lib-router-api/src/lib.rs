//! API router for the application's HTTP API.
//!
//! Provides the [`api_router`] function which constructs an [`axum::Router`]
//! with all API routes registered, including health-check endpoints.

mod health;

use axum::Router;
use axum::routing::get;

/// Internal application state shared across API handlers.
#[derive(Clone, Debug)]
struct AppState {}

/// Constructs the API [`Router`] with all registered routes.
///
/// # Routes
///
/// | Method | Path | Handler |
/// |--------|------|---------|
/// | `GET` | `/health/live` | [`health::live`] – liveness probe |
/// | `GET` | `/health/ready` | [`health::ready`] – readiness probe |
pub fn api_router() -> Router {
    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .with_state(AppState {})
}
