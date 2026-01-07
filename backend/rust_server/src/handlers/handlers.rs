use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    Extension,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Note, ShareRequest};
use crate::error::AppError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
}

pub async fn list_notes(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<Note>>, AppError> {
    let notes = sqlx::query_as!(
        Note,
        r#"
        SELECT id as "id!", owner_id as "owner_id!", title as "title!", content as "content!", locked_by, locked_at
        FROM notes n
        LEFT JOIN note_shares s ON n.id = s.note_id
        WHERE n.owner_id = $1 OR s.user_id = $1
        GROUP BY n.id, n.owner_id, n.title, n.content, n.locked_by, n.locked_at
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(notes))
}

pub async fn create_note(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    let note = sqlx::query_as!(
        Note,
        r#"
        INSERT INTO notes (owner_id, title, content) VALUES ($1, $2, $3) 
        RETURNING id as "id!", owner_id as "owner_id!", title as "title!", content as "content!", locked_by, locked_at
        "#,
        user_id,
        payload.title,
        payload.content
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn get_note(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(note_id): Path<Uuid>,
) -> Result<Json<Note>, AppError> {
    let note = sqlx::query_as!(
        Note,
        r#"
        SELECT id as "id!", owner_id as "owner_id!", title as "title!", content as "content!", locked_by, locked_at
        FROM notes n
        LEFT JOIN note_shares s ON n.id = s.note_id
        WHERE n.id = $1 AND (n.owner_id = $2 OR s.user_id = $2)
        "#,
        note_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| AppError::Unauthorized)?;

    Ok(Json(note))
}

// --- ADDING THE MISSING FUNCTIONS BELOW ---

pub async fn update_note(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(note_id): Path<Uuid>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<Note>, AppError> {
    let note = sqlx::query_as!(
        Note,
        r#"
        UPDATE notes SET title = $1, content = $2 
        WHERE id = $3 AND owner_id = $4 
        RETURNING id as "id!", owner_id as "owner_id!", title as "title!", content as "content!", locked_by, locked_at
        "#,
        payload.title,
        payload.content,
        note_id,
        user_id
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(note))
}

pub async fn lock_note(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Logic for locking goes here
    Ok(StatusCode::OK)
}

pub async fn share_note(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
    Json(_payload): Json<ShareRequest>,
) -> Result<StatusCode, AppError> {
    // Logic for sharing goes here
    Ok(StatusCode::OK)
}