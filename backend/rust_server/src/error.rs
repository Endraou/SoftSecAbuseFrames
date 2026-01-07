use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

pub enum AppError {
    SqlxError(sqlx::Error),
    Locked,
    Unauthorized,
    Internal(String),
}

// Convert common errors into AppError automatically
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::SqlxError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::SqlxError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Locked => (StatusCode::CONFLICT, "This note is currently locked by another user".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized access".to_string()),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

// Allow using ? on StatusCode in auth_handlers
impl From<StatusCode> for AppError {
    fn from(status: StatusCode) -> Self {
        match status {
            StatusCode::UNAUTHORIZED => Self::Unauthorized,
            _ => Self::Internal("An error occurred".to_string()),
        }
    }
}

// Allow converting (StatusCode, String) tuples into AppError
impl From<(StatusCode, String)> for AppError {
    fn from(tuple: (StatusCode, String)) -> Self {
        Self::Internal(tuple.1)
    }
}