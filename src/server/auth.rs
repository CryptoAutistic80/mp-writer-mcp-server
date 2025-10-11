use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::server::AppState;

const API_KEY_HEADER: &str = "x-api-key";

pub async fn require_api_key(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided = request
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok());

    match provided {
        Some(value) if value == state.api_key.as_ref() => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
