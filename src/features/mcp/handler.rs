use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};

use crate::core::error::AppError;
use crate::features::mcp::dto::{JsonRpcError, JsonRpcErrorResponse, JsonRpcRequest};
use crate::server::AppState;

pub async fn handle_mcp(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<JsonRpcRequest>(payload) {
        Ok(request) => match state.service.handle_jsonrpc(request).await {
            Ok(success) => Json(json!(success)).into_response(),
            Err(error) => Json(json!(error)).into_response(),
        },
        Err(err) => {
            let error = JsonRpcErrorResponse {
                jsonrpc: "2.0".to_string(),
                id: Value::Null,
                error: JsonRpcError {
                    code: -32700,
                    message: format!("failed to parse request: {err}"),
                    data: None,
                },
            };
            Json(json!(error)).into_response()
        }
    }
}

pub async fn handle_healthcheck() -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "status": "ok" })))
}
