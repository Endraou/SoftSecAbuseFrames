use ax_notes::{main as app_main, models::*, auth::Claims}; // On importe ton code
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt; 
use uuid::Uuid;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};

// --- UTILS DE TEST ---
// Note: Ces tests supposent que tu as une DB de test propre.

#[tokio::test]
async fn security_suite_abuse_frames() {
    let state = setup_test_state().await; // Initialisation fictive miaou
    let app = ax_notes::app(state); // On récupère ton Router complet

    // --- ABUSE FRAME 1: Stored XSS ---
    // On vérifie que le contenu est nettoyé avant d'être stocké miaou.
    let xss_payload = json!({"title": "XSS", "content": "<script>alert('miaou')</script>Normal"});
    let res = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/notes")
            .header("Authorization", format!("Bearer {}", valid_token()))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&xss_payload).unwrap()))
            .unwrap()
    ).await.unwrap();
    
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let saved_note: Note = serde_json::from_slice(&body).unwrap();
    assert!(!saved_note.content.contains("<script>"));

    // --- ABUSE FRAME 2: Reflected XSS ---
    // Le serveur doit renvoyer 404 ou valider proprement les routes miaou.
    let res = app.clone().oneshot(
        Request::builder().uri("/notes/<script>alert(1)</script>").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // --- ABUSE FRAME 5: IDOR ---
    // Un utilisateur ne peut pas lire la note d'un autre miaou.
    let res = app.clone().oneshot(
        Request::builder()
            .uri(format!("/notes/{}", note_id_user_a))
            .header("Authorization", format!("Bearer {}", token_user_b))
            .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // --- ABUSE FRAME 6: Élévation de Privilèges ---
    // Un utilisateur en lecture seule ne peut pas modifier la note miaou.
    let edit_payload = json!({"title": "Hack", "content": "tentative"});
    let res = app.clone().oneshot(
        Request::builder()
            .method("PUT")
            .uri(format!("/notes/{}", shared_note_id))
            .header("Authorization", format!("Bearer {}", token_read_only_user))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&edit_payload).unwrap()))
            .unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT); // Retourne Locked/Conflict si permission refusée miaou

    // --- ABUSE FRAME 7: Lock Bypass ---
    // On ne peut pas écrire sur une note verrouillée par quelqu'un d'autre miaou.
    let res = app.clone().oneshot(
        Request::builder()
            .method("PUT")
            .uri(format!("/notes/{}", locked_by_other_id))
            .header("Authorization", format!("Bearer {}", my_token))
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&edit_payload).unwrap()))
            .unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);

    // --- ABUSE FRAME 8: SQL Injection ---
    // Les requêtes préparées empêchent l'injection miaou.
    let sql_injection = json!({"username": "' OR 1=1 --", "password": "any"});
    let res = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&sql_injection).unwrap()))
            .unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // --- ABUSE FRAME 10: Replay Attack (Expiration) ---
    // Un token expiré après 20 minutes doit être refusé miaou.
    let expired_token = create_expired_token_for_test();
    let res = app.clone().oneshot(
        Request::builder()
            .uri("/notes")
            .header("Authorization", format!("Bearer {}", expired_token))
            .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // --- ABUSE FRAME 12 & 13: Brute Force & DoS ---
    // Le rate limiter bloque après trop de tentatives miaou.
    for _ in 0..10 {
        let _ = app.clone().oneshot(
            Request::builder().uri("/auth/login").method("POST").body(Body::empty()).unwrap()
        ).await.unwrap();
    }
    let res = app.clone().oneshot(
        Request::builder().uri("/auth/login").method("POST").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
}