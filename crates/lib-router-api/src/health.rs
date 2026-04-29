use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Liveness probe handler.
///
/// Returns `200 OK` to indicate the service process is alive.
pub(crate) async fn live() -> impl IntoResponse {
    StatusCode::OK
}

/// Readiness probe handler.
///
/// Returns `200 OK` to indicate the service is ready to accept traffic.
pub(crate) async fn ready() -> impl IntoResponse {
    StatusCode::OK
}
