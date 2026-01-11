use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// --- Note Models ---
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub content: String,
    pub locked_by: Option<Uuid>,
    pub locked_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct ShareRequest {
    pub username: String,
    pub can_write: bool,
}

// --- Auth Models (New) ---
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}