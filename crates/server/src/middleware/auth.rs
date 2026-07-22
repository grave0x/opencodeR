use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::OnceLock;

/// Leetopt: pre-computed expected Basic auth header value.
/// Stored as `"Basic " + base64("opencode:<password>")` — avoids decoding
/// and parsing on every request.
static EXPECTED_CREDENTIAL: OnceLock<String> = OnceLock::new();

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

/// Set the auth password and pre-compute the expected credential.
pub fn set_password(password: String) {
    // Leetopt: pre-compute the expected "Basic base64(opencode:password)" string
    // so the hot path does a single string comparison — no alloc, no decode.
    let cred = format!("opencode:{}", password);
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(cred.as_bytes());
    let expected = format!("Basic {}", encoded);
    let _ = EXPECTED_CREDENTIAL.set(expected);
}

pub fn is_enabled() -> bool {
    EXPECTED_CREDENTIAL.get().is_some()
}

/// Leetopt: single string comparison — no base64 decode, no UTF-8 validation,
/// no split, no allocation. The expected credential was pre-computed at setup.
pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let Some(expected) = EXPECTED_CREDENTIAL.get() else {
        return next.run(request).await;
    };

    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|val| val == expected.as_str());

    if authorized {
        next.run(request).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            [("www-authenticate", "Basic realm=\"Secure Area\"")],
            Json(ErrorResponse { message: "Authentication required".into() }),
        ).into_response()
    }
}
