use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    Extension,
};
use sqlx::Row;
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
    let notes = sqlx::query_as::<_, Note>(
        r#"
        SELECT 
            n.id, 
            n.owner_id, 
            n.title, 
            n.content, 
            n.locked_by, 
            n.locked_at,
            (n.owner_id = $1 OR EXISTS (
                SELECT 1 FROM note_shares 
                WHERE note_id = n.id AND user_id = $1 AND can_write = true
            )) as can_write
        FROM notes n
        LEFT JOIN note_shares s ON n.id = s.note_id
        WHERE n.owner_id = $1 OR s.user_id = $1
        GROUP BY n.id
        "#
    )
    .bind(user_id)
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
        RETURNING id as "id!", owner_id as "owner_id!", title as "title!", 
                content as "content!", locked_by, locked_at, 
                true as "can_write!" -- Add this line
        "#,
        user_id, payload.title, payload.content
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
        SELECT 
            n.id as "id!", 
            n.owner_id as "owner_id!", 
            n.title as "title!", 
            n.content as "content!", 
            n.locked_by, 
            n.locked_at, 
            (n.owner_id = $2 OR EXISTS (
                SELECT 1 FROM note_shares 
                WHERE note_id = $1 AND user_id = $2 AND can_write = true
            )) as "can_write!"
        FROM notes n
        LEFT JOIN note_shares s ON n.id = s.note_id
        WHERE n.id = $1 AND (n.owner_id = $2 OR s.user_id = $2)
        "#,
        note_id, user_id
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
    let result = sqlx::query_as::<_, Note>(
        r#"
        UPDATE notes 
        SET title = $1, content = $2, locked_by = NULL, locked_at = NULL
        WHERE id = $3 
        AND (
            owner_id = $4 OR 
            EXISTS (SELECT 1 FROM note_shares WHERE note_id = $3 AND user_id = $4 AND can_write = true)
        )
        AND (locked_by IS NULL OR locked_by = $4)
        RETURNING id, owner_id, title, content, locked_by, locked_at, true as can_write
        "#
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(note_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await?;

    match result {
        Some(note) => Ok(Json(note)),
        None => Err(AppError::Locked), // This matches your error.rs variant
    }
}

pub async fn delete_note(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(note_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND owner_id = $2")
        .bind(note_id)
        .bind(user_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Unauthorized);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn share_note(
    State(pool): State<PgPool>,
    Extension(current_user_id): Extension<Uuid>,
    Path(note_id): Path<Uuid>,
    Json(payload): Json<ShareRequest>,
) -> Result<StatusCode, AppError> {
    // 1. Vérifier la propriété (Sécurité avant tout miaou)
    let row = sqlx::query("SELECT owner_id FROM notes WHERE id = $1")
        .bind(note_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Internal("Note introuvable".into()))?;

    let owner_id: Uuid = row.get("owner_id");
    if owner_id != current_user_id {
        return Err(AppError::Unauthorized);
    }

    // 2. Trouver l'ID du destinataire par son nom
    let target_row = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Internal("Utilisateur introuvable".into()))?;

    let target_id: Uuid = target_row.get("id");

    if target_id == current_user_id {
        return Err(AppError::Internal("Vous possédez déjà cette note".into()));
    }

    // 3. Insérer le partage
    sqlx::query(
        "INSERT INTO note_shares (note_id, user_id, can_write) VALUES ($1, $2, $3) ON CONFLICT (note_id, user_id) DO UPDATE SET can_write = $3"
    )
    .bind(note_id)
    .bind(target_id)
    .bind(payload.can_write)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

// Logic for checking access
async fn check_access(pool: &PgPool, user_id: &Uuid, note_id: &Uuid, require_write: bool) -> Result<bool, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT can_write FROM note_shares 
        WHERE note_id = $1 AND user_id = $2
        UNION
        SELECT true FROM notes WHERE id = $1 AND owner_id = $2
        "#,
        note_id, user_id
    )
    .fetch_optional(pool)
    .await?;

    match row {
    // Use unwrap_or(false) to convert Option<bool> to bool
    Some(r) => if require_write { Ok(r.can_write.unwrap_or(false)) } else { Ok(true) },
    None => Ok(false)
}
}

pub async fn lock_note(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(note_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // 1. Check if user has write permission
    if !check_access(&pool, &user_id, &note_id, true).await? {
        return Err(AppError::Forbidden);
    }

    // 2. Try to acquire lock if not already locked or if lock is expired (e.g., > 15 mins)
    let result = sqlx::query!(
        r#"
        UPDATE notes 
        SET locked_by = $1, locked_at = NOW()
        WHERE id = $2 AND (locked_by IS NULL OR locked_by = $1 OR locked_at < NOW() - INTERVAL '15 minutes')
        "#,
        user_id, note_id
    )
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict); // Someone else is editing
    }

    Ok(StatusCode::OK)
}

pub async fn unlock_note(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<Uuid>,
    Path(note_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!(
        r#"
        UPDATE notes 
        SET locked_by = NULL, locked_at = NULL
        WHERE id = $1 AND (locked_by = $2 OR owner_id = $2)
        "#,
        note_id, user_id
    )
    .execute(&pool)
    .await?; // Just use ? here, SqlxError is already handled by From implementation

    if result.rows_affected() == 0 {
        return Ok(StatusCode::NOT_MODIFIED);
    }

    Ok(StatusCode::OK)
}