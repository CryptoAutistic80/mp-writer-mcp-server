use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde_json::{Value, json};

use crate::core::error::AppError;
use crate::features::mcp::dto::{JsonRpcError, JsonRpcErrorResponse, JsonRpcRequest};
use crate::server::AppState;

pub async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<JsonRpcRequest>(payload) {
        Ok(request) => {
            let header_protocol_version = headers
                .get("MCP-Protocol-Version")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.to_string());

            match state
                .service
                .handle_jsonrpc(request, header_protocol_version)
                .await
            {
                Ok(Some(success)) => Json(json!(success)).into_response(),
                Ok(None) => StatusCode::NO_CONTENT.into_response(),
                Err(error) => Json(json!(error)).into_response(),
            }
        }
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
