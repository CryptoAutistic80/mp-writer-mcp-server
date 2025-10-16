use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {message}")]
    Configuration { message: String },
    #[error("bad request: {message}")]
    BadRequest { message: String },
    #[error("upstream error: {message}")]
    Upstream {
        message: String,
        data: Option<Value>,
    },
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl AppError {
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    pub fn upstream_with_data(message: impl Into<String>, data: Value) -> Self {
        Self::Upstream {
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Configuration { message } | Self::Internal { message } => {
                (StatusCode::INTERNAL_SERVER_ERROR, message)
            }
            Self::BadRequest { message } => (StatusCode::BAD_REQUEST, message),
            Self::Upstream { message, .. } => (StatusCode::BAD_GATEWAY, message),
        };

        let body = Json(ErrorResponse { error: message });

        (status, body).into_response()
    }
}
