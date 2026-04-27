//! Web router for the maintenance list application.

use axum::Router;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use lib_web::{get_file_data, get_index_data};

/// Returns `true` if the request signals it accepts gzip-encoded responses.
fn accepts_gzip(request_headers: &HeaderMap) -> bool {
    request_headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| {
            s.split(',')
                .any(|token| token.split(';').next().is_some_and(|t| t.trim() == "gzip"))
        })
}

/// Handles web asset requests by serving static files embedded in the binary.
///
/// Resolves the requested path to a file, sets appropriate security headers, and returns either
/// gzip-compressed or uncompressed content based on the `Accept-Encoding` request header.
/// Falls back to serving `index.html` when no path is provided or the requested file is not found.
async fn web_route(request_headers: HeaderMap, path: Option<Path<String>>) -> impl IntoResponse {
    let path_string = match path {
        Some(Path(path)) => path,
        None => "index.html".to_string(),
    };

    let file_data = get_file_data(path_string.as_str()).unwrap_or(get_index_data());

    let mut headers = HeaderMap::new();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Vite outputs fingerprinted assets under `assets/`; these are safe to cache indefinitely.
    // Everything else (including index.html) must be revalidated on every request.
    let cache_control = if path_string.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );

    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; font-src 'self'; img-src 'self' data:; \
        script-src 'self'; style-src 'self'; connect-src 'self'; \
        frame-src 'self'; frame-ancestors 'self'; form-action 'self';",
        ),
    );
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));

    let content_type = if file_data.mime_type.is_empty() {
        "application/octet-stream"
    } else {
        file_data.mime_type
    };
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

    if accepts_gzip(&request_headers) {
        headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        (headers, file_data.data_gzip).into_response()
    } else {
        (headers, file_data.data_uncompressed).into_response()
    }
}

/// Creates an [`axum::Router`] that serves all web application assets.
///
/// Registers two routes:
/// - `/{*path}` — serves any static asset by its path.
/// - `/` — serves the application root (`index.html`).
pub fn web_router() -> Router {
    Router::new()
        .route("/{*path}", get(web_route))
        .route("/", get(web_route))
}

/// Serves the embedded `favicon.ico` file.
///
/// Returns `404 Not Found` if the file is not present in the embedded assets,
/// unlike [`web_route`] which would fall back to `index.html`.
async fn favicon_route(request_headers: HeaderMap) -> impl IntoResponse {
    match get_file_data("favicon.ico") {
        Some(file_data) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );

            let content_type = if file_data.mime_type.is_empty() {
                "application/octet-stream"
            } else {
                file_data.mime_type
            };
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
            headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));

            if accepts_gzip(&request_headers) {
                headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
                (StatusCode::OK, headers, file_data.data_gzip).into_response()
            } else {
                (StatusCode::OK, headers, file_data.data_uncompressed).into_response()
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Creates an [`axum::Router`] that serves the application favicon.
///
/// Registers a single route:
/// - `/favicon.ico` — serves the embedded favicon file, or `404 Not Found` if not embedded.
pub fn favicon_router() -> Router {
    Router::new().route("/favicon.ico", get(favicon_route))
}
