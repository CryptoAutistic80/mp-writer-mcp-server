use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::http::{HeaderMap, StatusCode};
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
            if let Some(expected_version) = state.service.negotiated_protocol_version() {
                let header_value = headers
                    .get("MCP-Protocol-Version")
                    .and_then(|value| value.to_str().ok());

                let header_valid = header_value
                    .map(|provided| provided == expected_version)
                    .unwrap_or(false);

                if !header_valid {
                    let message = match header_value {
                        Some(provided) => format!(
                            "MCP-Protocol-Version header mismatch: expected {expected_version}, received {provided}"
                        ),
                        None => format!(
                            "MCP-Protocol-Version header missing; expected {expected_version}"
                        ),
                    };

                    let error = JsonRpcErrorResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.clone().unwrap_or(Value::Null),
                        error: JsonRpcError {
                            code: -32600,
                            message,
                            data: None,
                        },
                    };

                    return Json(json!(error)).into_response();
                }
            }

            match state.service.handle_jsonrpc(request).await {
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
