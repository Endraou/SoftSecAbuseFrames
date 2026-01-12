use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

pub enum AppError {
    SqlxError(sqlx::Error),
    Locked,           // Used for general locking logic
    Unauthorized,     // 401: Identity is unknown
    Forbidden,        // 403: Identity known but permission denied
    Conflict,         // 409: Resource state conflict (e.g., someone else has the lock)
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::SqlxError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::SqlxError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Locked => (StatusCode::CONFLICT, "Note is locked".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized access".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "You do not have permission to do this".to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, "This note is currently being edited by someone else".to_string()),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

// Helper to handle strings as Internal errors
impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::Internal(message)
    }
}

// Add this to your error.rs to fix the trait bound error
impl From<axum::http::StatusCode> for AppError {
    fn from(status: axum::http::StatusCode) -> Self {
        match status {
            axum::http::StatusCode::UNAUTHORIZED => Self::Unauthorized,
            axum::http::StatusCode::FORBIDDEN => Self::Forbidden,
            axum::http::StatusCode::CONFLICT => Self::Conflict,
            _ => Self::Internal("An error occurred".to_string()),
        }
    }
}