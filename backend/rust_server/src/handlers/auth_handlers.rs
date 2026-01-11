use axum::{extract::State, Json};
use sqlx::PgPool;
use crate::auth::{hash_password, verify_password, create_jwt};
use crate::models::{LoginRequest, AuthResponse, RegisterRequest};
use crate::error::AppError;

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let hashed = hash_password(&payload.password);
    
    // Use double quotes \"id!\" for the alias
    let user = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id as \"id!\"",
        payload.username, 
        hashed
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| AppError::Internal("User already exists or DB error".into()))?;

    let token = create_jwt(user.id).map_err(|s| AppError::from(s))?;
    Ok(Json(AuthResponse { token }))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Use double quotes \"id!\" for the alias
    let user = sqlx::query!(
        "SELECT id as \"id!\", password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| AppError::Internal("Database error".into()))?
    .ok_or(AppError::Unauthorized)?;

    if verify_password(&payload.password, &user.password_hash) {
        let token = create_jwt(user.id).map_err(|s| AppError::from(s))?;
        Ok(Json(AuthResponse { token }))
    } else {
        Err(AppError::Unauthorized)
    }
}